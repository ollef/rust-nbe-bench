use crate::index::Index;

#[derive(Clone, Debug)]

pub enum Term<'a> {
    Variable(Index),
    Lambda(TermRef<'a>),
    Application(TermRef<'a>, TermRef<'a>),
}

pub type Type<'a> = Term<'a>;
pub type TermRef<'a> = &'a Term<'a>;
pub type TypeRef<'a> = &'a Type<'a>;

pub struct Builder {
    arena: blink_alloc::Blink,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            arena: blink_alloc::Blink::new(),
        }
    }
    pub fn variable<'a>(&'a self, index: Index) -> TermRef<'a> {
        self.arena.put_no_drop(Term::Variable(index))
    }

    pub fn lambda<'a>(&'a self, body: TermRef<'a>) -> TermRef<'a> {
        self.arena.put_no_drop(Term::Lambda(body))
    }

    pub fn application<'a>(&'a self, function: TermRef<'a>, argument: TermRef<'a>) -> TermRef<'a> {
        self.arena
            .put_no_drop(Term::Application(function, argument))
    }

    pub fn v<'a>(&'a self, index: usize) -> TermRef<'a> {
        self.variable(Index(index))
    }

    pub fn apps<'a>(&'a self, f: TermRef<'a>, args: &[TermRef<'a>]) -> TermRef<'a> {
        args.iter().fold(f, |f, arg| self.application(f, arg))
    }

    pub fn l<'a>(&'a self, body: TermRef<'a>) -> TermRef<'a> {
        self.lambda(body)
    }
}
