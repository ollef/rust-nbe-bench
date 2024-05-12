use crate::{
    index::{Index, Level},
    syntax::{self, Term, TermRef},
};
use rclite::Rc;

#[derive(Clone)]
pub enum Head {
    Variable(Level),
}

pub struct Spine<'a> {
    reversed_values: Vec<ValueRef<'a>>,
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

    pub fn into_iter<'s>(self) -> impl Iterator<Item = ValueRef<'a>> {
        self.reversed_values.into_iter().rev()
    }

    pub fn is_empty(&self) -> bool {
        self.reversed_values.is_empty()
    }
}

pub struct Closure<'a> {
    term: TermRef<'a>,
    environment: Environment<'a>,
}

pub enum Value<'a> {
    Neutral {
        head: Head,
        spine: Vec<ValueRef<'a>>,
    },
    Lambda(Closure<'a>),
}

pub type Type<'a> = Value<'a>;
pub type ValueRef<'a> = Rc<Value<'a>>;
pub type TypeRef<'a> = Rc<Type<'a>>;

#[derive(Clone)]
pub struct Environment<'a> {
    values: Vec<ValueRef<'a>>,
}

impl<'a> std::ops::Index<Index> for Environment<'a> {
    type Output = ValueRef<'a>;

    fn index(&self, index: Index) -> &Self::Output {
        &self.values[self.values.len() - index.to_int() - 1]
    }
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Environment { values: Vec::new() }
    }

    pub fn extend(&mut self, value: ValueRef<'a>) {
        self.values.push(value)
    }

    pub fn local<F, A>(&mut self, f: F) -> A
    where
        F: FnOnce(&mut Self) -> A,
    {
        let len_before = self.values.len();
        let result = f(self);
        assert!(self.values.len() >= len_before);
        self.values.truncate(len_before);
        result
    }
}

impl<'a> Value<'a> {
    pub fn variable(level: Level) -> ValueRef<'a> {
        Rc::new(Value::Neutral {
            head: Head::Variable(level),
            spine: Vec::new(),
        })
    }
}

pub fn apply<'a>(function: &Value<'a>, argument: ValueRef<'a>) -> ValueRef<'a> {
    match function {
        Value::Neutral { head, spine } => Rc::new(Value::Neutral {
            head: head.clone(),
            spine: Vec::from_iter(spine.iter().cloned().chain(std::iter::once(argument))),
        }),
        Value::Lambda(Closure { term, environment }) => {
            let mut environment = environment.clone();
            environment.extend(argument);
            term.evaluate_rc(&mut environment)
        }
    }
}

pub fn apply_spine<'a>(function: &ValueRef<'a>, mut spine: Spine<'a>) -> ValueRef<'a> {
    if spine.is_empty() {
        return function.clone();
    }
    match &**function {
        Value::Neutral {
            head,
            spine: function_spine,
        } => {
            let mut new_spine = function_spine.clone();
            new_spine.extend(spine.into_iter());
            Rc::new(Value::Neutral {
                head: head.clone(),
                spine: new_spine,
            })
        }
        Value::Lambda(Closure { term, environment }) => {
            if let Some(argument) = spine.pop_front() {
                let mut environment = environment.clone();
                environment.extend(argument);
                term.evaluate_with_spine_rc(spine, &mut environment)
            } else {
                function.clone()
            }
        }
    }
}

impl<'a> Term<'a> {
    pub fn evaluate_rc(&self, environment: &mut Environment<'a>) -> ValueRef<'a> {
        self.evaluate_with_spine_rc(Spine::new(), environment)
    }

    pub fn evaluate_with_spine_rc(
        &self,
        mut spine: Spine<'a>,
        environment: &mut Environment<'a>,
    ) -> ValueRef<'a> {
        let mut head = self;
        loop {
            match head {
                Term::Variable(index) => {
                    let head = &environment[*index];
                    return apply_spine(head, spine);
                }
                Term::Lambda(body) => {
                    if let Some(argument) = spine.pop_front() {
                        environment.extend(argument);
                        head = body;
                    } else {
                        return Rc::new(Value::Lambda(Closure {
                            term: body,
                            environment: environment.clone(),
                        }));
                    }
                }
                Term::Application(function, argument) => {
                    let argument =
                        environment.local(|environment| argument.evaluate_rc(environment));
                    spine.push_front(argument);
                    head = function;
                }
            }
        }
    }
}

impl<'a> Value<'a> {
    pub fn quote(&self, level: Level, syntax_builder: &'a syntax::Builder) -> syntax::TermRef<'a> {
        match self {
            Value::Neutral { head, spine } => head.quote(spine, level, syntax_builder),
            Value::Lambda(Closure { term, environment }) => syntax_builder.lambda({
                let mut environment = environment.clone();
                environment.extend(Value::variable(level));
                term.evaluate_rc(&mut environment)
                    .quote(level + 1, syntax_builder)
            }),
        }
    }
}

impl Head {
    pub fn quote<'a>(
        &self,
        spine: &Vec<ValueRef<'a>>,
        level: Level,
        syntax_builder: &'a syntax::Builder,
    ) -> syntax::TermRef<'a> {
        let mut result = match self {
            Head::Variable(var_level) => syntax_builder.variable(var_level.to_index(level)),
        };
        for arg in spine.iter() {
            result = syntax_builder.application(result, arg.quote(level, syntax_builder));
        }
        result
    }
}
