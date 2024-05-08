use crate::syntax::{Index, Literal, Term, TermRef};
use rclite::Rc;

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
    Literal(Literal),
    Lambda(TypeRef<'a>, Closure<'a>),
    Pi(TypeRef<'a>, Closure<'a>),
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
        &self.values[index.int]
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

pub fn apply<'a>(function: &Value<'a>, argument: ValueRef<'a>) -> ValueRef<'a> {
    match function {
        Value::Neutral { head, spine } => Rc::new(Value::Neutral {
            head: head.clone(),
            spine: Vec::from_iter(spine.iter().cloned().chain(std::iter::once(argument))),
        }),
        Value::Literal(_) => panic!("Applying literal"),
        Value::Lambda(_type, Closure { term, environment }) => environment
            .clone()
            .extend(argument, |environment| evaluate(term, environment)),
        Value::Pi(_, _) => panic!("Applying pi"),
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
        Value::Literal(literal) => {
            panic!("Applying literal")
        }
        Value::Lambda(_type, Closure { term, environment }) => {
            if let Some(argument) = spine.pop_front() {
                environment.clone().extend(argument, |environment| {
                    evaluate_with_spine(term, spine, environment)
                })
            } else {
                function.clone()
            }
        }
        Value::Pi(_, _) => {
            panic!("Applying pi")
        }
    }
}

pub fn evaluate<'a>(term: TermRef<'a>, environment: &mut Environment<'a>) -> ValueRef<'a> {
    evaluate_with_spine(term, Spine::new(), environment)
}

pub fn evaluate_with_spine<'a>(
    term: TermRef<'a>,
    mut spine: Spine<'a>,
    environment: &mut Environment<'a>,
) -> ValueRef<'a> {
    match term {
        Term::Variable(index) => {
            let head = &environment[*index];
            apply_spine(head, spine)
        }
        Term::Literal(literal) => {
            assert!(spine.is_empty());
            Rc::new(Value::Literal(literal.clone()))
        }
        Term::Lambda(type_, body) => {
            if let Some(argument) = spine.pop_front() {
                environment.extend(argument, |environment| {
                    evaluate_with_spine(body, spine, environment)
                })
            } else {
                let type_ = evaluate(type_, environment);
                Rc::new(Value::Lambda(
                    type_,
                    Closure {
                        term: body,
                        environment: environment.clone(),
                    },
                ))
            }
        }
        Term::Pi(domain, target) => {
            assert!(spine.is_empty());
            let domain = evaluate(domain, environment);
            let target_closure = Closure {
                environment: environment.clone(),
                term: target,
            };
            Rc::new(Value::Pi(domain, target_closure))
        }
        Term::Application(function, argument) => {
            let argument = evaluate(argument, environment);
            spine.push_front(argument);
            evaluate_with_spine(function, spine, environment)
        }
    }
}
