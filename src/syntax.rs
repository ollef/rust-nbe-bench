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
}
