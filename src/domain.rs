use crate::syntax::{Index, Literal, Term, TermRef};

#[derive(Clone, Copy)]
pub struct Variable {
    int: usize,
}

#[derive(Clone)]
pub enum Head {
    Variable(Variable),
}

pub struct Neutral<'a> {
    head: Head,
    spine: Spine<'a>,
}

pub struct Spine<'a> {
    values: Vec<ValueRef<'a>>,
}

pub struct ConsSpine<'a> {
    reversed_values: Vec<ValueRef<'a>>,
}

pub struct ConstantSpine<'a> {
    values: &'a [ValueRef<'a>],
}

pub struct Closure<'a> {
    term: TermRef<'a>,
    environment: ConstantEnvironment<'a>,
}

pub enum Value<'a> {
    Neutral { head: Head, spine: Spine<'a> },
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

    pub fn neutral<'a>(&'a self, head: Head, spine: Spine<'a>) -> ValueRef<'a> {
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
        Value::Neutral { head, spine } => {
            let mut spine = spine.clone();
            spine.push(argument);
            builder.neutral(head.clone(), spine)
        }
        Value::Literal(_) => panic!("Applying literal"),
        Value::Lambda(_type, Closure { term, environment }) => Environment::from(environment)
            .extend(argument, |environment| evaluate(term, environment, builder)),
        Value::Pi(_, _) => panic!("Applying pi"),
    }
}

pub fn apply_spine<'a>(
    function: ValueRef<'a>,
    mut reversed_spine: Spine<'a>,
    builder: &'a Builder,
) -> ValueRef<'a> {
    match function {
        Value::Neutral { head, spine } => {
            reversed_spine.reverse();
            let mut spine = spine.clone();
            spine.append(&mut reversed_spine);
            builder.neutral(head.clone(), spine)
        }
        Value::Literal(literal) => {
            assert!(reversed_spine.is_empty());
            builder.literal(literal.clone())
        }
        Value::Lambda(_type, Closure { term, environment }) => {
            if let Some(argument) = reversed_spine.pop() {
                Environment::from(environment).extend(argument, |environment| {
                    evaluate_with_spine(term, reversed_spine, environment, builder)
                })
            } else {
                function
            }
        }
        Value::Pi(_, _) => todo!(),
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
    mut reversed_spine: Spine<'a>,
    environment: &mut Environment<'a>,
    builder: &'a Builder,
) -> ValueRef<'a> {
    match term {
        Term::Variable(index) => {
            let head = environment[*index];
            apply_spine(head, reversed_spine, builder)
        }
        Term::Literal(literal) => {
            assert!(reversed_spine.is_empty());
            builder.literal(literal.clone())
        }
        Term::Lambda(type_, body) => {
            if let Some(argument) = reversed_spine.pop() {
                environment.extend(argument, |environment| {
                    evaluate_with_spine(body, reversed_spine, environment, builder)
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
        Term::Pi(_, _) => todo!(),
        Term::Application(function, argument) => {
            let argument = evaluate(argument, environment, builder);
            reversed_spine.push(argument);
            evaluate_with_spine(function, reversed_spine, environment, builder)
        }
    }
}
