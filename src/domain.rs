use crate::syntax::{Index, Literal, Term, TermRef};

#[derive(Clone, Copy)]
pub struct Variable {
    int: usize,
}

#[derive(Clone)]
pub enum Head {
    Variable(Variable),
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
    Literal(Literal),
    Lambda(TypeRef<'a>, Closure<'a>),
    Pi(TypeRef<'a>, Closure<'a>),
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
        &self.values[index.int]
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
    pub fn variable<'a>(&'a self, variable: Variable) -> ValueRef<'a> {
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

    pub fn literal<'a>(&'a self, literal: Literal) -> ValueRef<'a> {
        self.arena.put(Value::Literal(literal))
    }

    pub fn lambda<'a>(&'a self, type_: TypeRef<'a>, body: Closure<'a>) -> ValueRef<'a> {
        self.arena.put_no_drop(Value::Lambda(type_, body))
    }

    pub fn pi<'a>(&'a self, domain: TypeRef<'a>, target: Closure<'a>) -> ValueRef<'a> {
        self.arena.put_no_drop(Value::Pi(domain, target))
    }
}

impl<'a> Environment<'a> {
    pub fn extend<F, A>(&mut self, value: ValueRef<'a>, f: F) -> A
    where
        F: FnOnce(&mut Self) -> A,
    {
        self.values.push(value);
        let result = f(self);
        self.values.pop();
        result
    }
}

pub fn apply<'a>(
    function: ValueRef<'a>,
    argument: ValueRef<'a>,
    builder: &'a Builder,
) -> ValueRef<'a> {
    match function {
        Value::Neutral { head, spine } => builder.neutral(
            head.clone(),
            ConstantSpine::from_iter(spine.iter().chain(std::iter::once(argument)), builder),
        ),
        Value::Literal(_) => panic!("Applying literal"),
        Value::Lambda(_type, Closure { term, environment }) => Environment::from(environment)
            .extend(argument, |environment| evaluate(term, environment, builder)),
        Value::Pi(_, _) => panic!("Applying pi"),
    }
}

pub fn apply_spine<'a>(
    function: ValueRef<'a>,
    mut spine: Spine<'a>,
    builder: &'a Builder,
) -> ValueRef<'a> {
    if spine.is_empty() {
        return function;
    }
    match function {
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
        Value::Literal(literal) => {
            panic!("Applying literal")
        }
        Value::Lambda(_type, Closure { term, environment }) => {
            if let Some(argument) = spine.pop_front() {
                Environment::from(environment).extend(argument, |environment| {
                    evaluate_with_spine(term, spine, environment, builder)
                })
            } else {
                function
            }
        }
        Value::Pi(_, _) => {
            panic!("Applying pi")
        }
    }
}

pub fn evaluate<'a>(
    term: TermRef<'a>,
    environment: &mut Environment<'a>,
    builder: &'a Builder,
) -> ValueRef<'a> {
    evaluate_with_spine(term, Spine::new(), environment, builder)
}

pub fn evaluate_with_spine<'a>(
    term: TermRef<'a>,
    mut spine: Spine<'a>,
    environment: &mut Environment<'a>,
    builder: &'a Builder,
) -> ValueRef<'a> {
    match term {
        Term::Variable(index) => {
            let head = environment[*index];
            apply_spine(head, spine, builder)
        }
        Term::Literal(literal) => {
            assert!(spine.is_empty());
            builder.literal(literal.clone())
        }
        Term::Lambda(type_, body) => {
            if let Some(argument) = spine.pop_front() {
                environment.extend(argument, |environment| {
                    evaluate_with_spine(body, spine, environment, builder)
                })
            } else {
                let type_ = evaluate(type_, environment, builder);
                builder.lambda(
                    type_,
                    Closure {
                        term: body,
                        environment: ConstantEnvironment::from(environment, builder),
                    },
                )
            }
        }
        Term::Pi(domain, target) => {
            assert!(spine.is_empty());
            let domain = evaluate(domain, environment, builder);
            let target_closure = Closure {
                environment: ConstantEnvironment::from(environment, builder),
                term: target,
            };
            builder.pi(domain, target_closure)
        }
        Term::Application(function, argument) => {
            let argument = evaluate(argument, environment, builder);
            spine.push_front(argument);
            evaluate_with_spine(function, spine, environment, builder)
        }
    }
}
