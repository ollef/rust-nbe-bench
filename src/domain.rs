use crate::{
    index::{Index, Level},
    syntax::{self, Term, TermRef},
};

#[derive(Clone)]
pub enum Head {
    Variable(Level),
}

pub struct Spine<'a> {
    reversed_values: Vec<ValueRef<'a>>,
}

pub struct ConstantSpine<'a> {
    values: &'a [ValueRef<'a>],
}

impl<'a> ConstantSpine<'a> {
    pub fn from_iter<It>(iter: It, builder: &'a Builder) -> Self
    where
        It: Iterator<Item = ValueRef<'a>>,
    {
        ConstantSpine {
            values: builder.arena.emplace_no_drop().from_iter(iter),
        }
    }

    pub fn from_spine(spine: &'a Spine<'a>, builder: &'a Builder) -> Self {
        Self::from_iter(spine.iter().copied(), builder)
    }

    pub fn iter(&self) -> impl Iterator<Item = ValueRef<'a>> {
        self.values.iter().copied()
    }
}

impl<'a> From<&ConstantSpine<'a>> for Spine<'a> {
    fn from(spine: &ConstantSpine<'a>) -> Self {
        Spine {
            reversed_values: Vec::from_iter(spine.values.iter().rev().map(|&v| v)),
        }
    }
}

impl<'a> Spine<'a> {
    pub fn new() -> Self {
        Spine {
            reversed_values: Vec::new(),
        }
    }

    pub fn push_front(&mut self, value: ValueRef<'a>) {
        self.reversed_values.push(value)
    }

    pub fn pop_front(&mut self) -> Option<ValueRef<'a>> {
        self.reversed_values.pop()
    }

    pub fn iter<'s>(&'s self) -> impl Iterator<Item = &'s ValueRef<'a>> {
        self.reversed_values.iter().rev()
    }

    pub fn is_empty(&self) -> bool {
        self.reversed_values.is_empty()
    }
}

pub struct Closure<'a> {
    term: TermRef<'a>,
    environment: ConstantEnvironment<'a>,
}

pub enum Value<'a> {
    Neutral {
        head: Head,
        spine: ConstantSpine<'a>,
    },
    Lambda(Closure<'a>),
}

pub type Type<'a> = Value<'a>;
pub type ValueRef<'a> = &'a Value<'a>;
pub type TypeRef<'a> = &'a Type<'a>;

#[derive(Clone)]
pub struct Environment<'a> {
    values: Vec<ValueRef<'a>>,
}

pub struct ConstantEnvironment<'a> {
    values: &'a [ValueRef<'a>],
}

pub struct Builder {
    arena: blink_alloc::Blink,
}

impl<'a> std::ops::Index<Index> for Environment<'a> {
    type Output = ValueRef<'a>;

    fn index(&self, index: Index) -> &Self::Output {
        &self.values[index.to_int()]
    }
}

impl<'a> From<&ConstantEnvironment<'a>> for Environment<'a> {
    fn from(environment: &ConstantEnvironment<'a>) -> Self {
        Environment {
            values: environment.values.into(),
        }
    }
}

impl<'a> ConstantEnvironment<'a> {
    fn from(environment: &Environment<'a>, builder: &'a Builder) -> Self {
        ConstantEnvironment {
            values: builder
                .arena
                .emplace_no_drop()
                .from_iter(environment.values.iter().map(|&v| v)),
        }
    }
}

impl Builder {
    pub fn variable<'a>(&'a self, variable: Level) -> ValueRef<'a> {
        self.neutral(
            Head::Variable(variable),
            ConstantSpine {
                values: self.arena.put_no_drop([]),
            },
        )
    }

    pub fn neutral<'a>(&'a self, head: Head, spine: ConstantSpine<'a>) -> ValueRef<'a> {
        self.arena.put_no_drop(Value::Neutral { head, spine })
    }

    pub fn lambda<'a>(&'a self, body: Closure<'a>) -> ValueRef<'a> {
        self.arena.put_no_drop(Value::Lambda(body))
    }
}

impl<'a> Environment<'a> {
    pub fn extend(&mut self, value: ValueRef<'a>) {
        self.values.push(value)
    }

    pub fn local<F, Result>(&mut self, f: F) -> Result
    where
        F: FnOnce(&mut Self) -> Result,
    {
        let len_before = self.values.len();
        let result = f(self);
        assert!(self.values.len() >= len_before);
        self.values.truncate(len_before);
        result
    }
}

impl<'a> Value<'a> {
    pub fn apply(&self, argument: ValueRef<'a>, builder: &'a Builder) -> ValueRef<'a> {
        match self {
            Value::Neutral { head, spine } => builder.neutral(
                head.clone(),
                ConstantSpine::from_iter(spine.iter().chain(std::iter::once(argument)), builder),
            ),
            Value::Lambda(Closure { term, environment }) => {
                let mut environment = Environment::from(environment);
                environment.extend(argument);
                term.evaluate(&mut environment, builder)
            }
        }
    }

    pub fn apply_spine(
        self: ValueRef<'a>,
        mut spine: Spine<'a>,
        builder: &'a Builder,
    ) -> ValueRef<'a> {
        if spine.is_empty() {
            return self;
        }
        match self {
            Value::Neutral {
                head,
                spine: function_spine,
            } => {
                let spine = ConstantSpine::from_iter(
                    function_spine.iter().chain(spine.iter().copied()),
                    builder,
                );
                builder.neutral(head.clone(), spine)
            }
            Value::Lambda(Closure { term, environment }) => {
                if let Some(argument) = spine.pop_front() {
                    let mut environment = Environment::from(environment);
                    environment.extend(argument);
                    term.evaluate_with_spine(spine, &mut environment, builder)
                } else {
                    self
                }
            }
        }
    }
}

impl<'a> Term<'a> {
    pub fn evaluate(
        &self,
        environment: &mut Environment<'a>,
        builder: &'a Builder,
    ) -> ValueRef<'a> {
        self.evaluate_with_spine(Spine::new(), environment, builder)
    }

    pub fn evaluate_with_spine(
        &self,
        mut spine: Spine<'a>,
        environment: &mut Environment<'a>,
        builder: &'a Builder,
    ) -> ValueRef<'a> {
        match self {
            Term::Variable(index) => {
                let head = environment[*index];
                head.apply_spine(spine, builder)
            }
            Term::Lambda(body) => {
                if let Some(argument) = spine.pop_front() {
                    environment.extend(argument);
                    body.evaluate_with_spine(spine, environment, builder)
                } else {
                    builder.lambda(Closure {
                        term: body,
                        environment: ConstantEnvironment::from(environment, builder),
                    })
                }
            }
            Term::Application(function, argument) => {
                let argument =
                    environment.local(|environment| argument.evaluate(environment, builder));
                spine.push_front(argument);
                function.evaluate_with_spine(spine, environment, builder)
            }
        }
    }
}

impl<'a> Value<'a> {
    pub fn quote(
        &self,
        level: Level,
        builder: &'a Builder,
        syntax_builder: &'a syntax::Builder,
    ) -> syntax::TermRef<'a> {
        match self {
            Value::Neutral { head, spine } => head.quote(spine, level, builder, syntax_builder),
            Value::Lambda(closure) => syntax_builder.lambda({
                let mut environment = Environment::from(&closure.environment);
                environment.extend(builder.variable(level));
                closure.term.evaluate(&mut environment, builder).quote(
                    level + 1,
                    builder,
                    syntax_builder,
                )
            }),
        }
    }
}

impl Head {
    pub fn quote<'a>(
        &self,
        spine: &ConstantSpine<'a>,
        level: Level,
        builder: &'a Builder,
        syntax_builder: &'a syntax::Builder,
    ) -> syntax::TermRef<'a> {
        let mut result = match self {
            Head::Variable(var_level) => syntax_builder.variable(var_level.to_index(level)),
        };
        for arg in spine.iter() {
            result = syntax_builder.application(result, arg.quote(level, builder, syntax_builder));
        }
        result
    }
}
