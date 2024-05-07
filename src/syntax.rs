use typed_arena::Arena;

#[derive(Copy, Clone)]
pub struct Index {
    pub int: usize,
}

#[derive(Clone)]
pub enum Literal {
    Int(i64),
}

pub enum Term<'a> {
    Variable(Index),
    Literal(Literal),
    Lambda(TypeRef<'a>, TermRef<'a>),
    Pi(TypeRef<'a>, TermRef<'a>),
    Application(TermRef<'a>, TermRef<'a>),
}

pub type Type<'a> = Term<'a>;
pub type TermRef<'a> = &'a Term<'a>;
pub type TypeRef<'a> = &'a Type<'a>;

pub struct Builder<'a> {
    arena: Arena<Term<'a>>,
}

impl<'a> Builder<'a> {
    pub fn variable(&'a mut self, index: Index) -> TermRef<'a> {
        self.arena.alloc(Term::Variable(index))
    }

    pub fn literal(&'a mut self, literal: Literal) -> TermRef<'a> {
        self.arena.alloc(Term::Literal(literal))
    }

    pub fn lambda(&'a mut self, type_: TypeRef<'a>, body: TermRef<'a>) -> TermRef<'a> {
        self.arena.alloc(Term::Lambda(type_, body))
    }

    pub fn pi(&'a mut self, domain: TypeRef<'a>, target: TermRef<'a>) -> TermRef<'a> {
        self.arena.alloc(Term::Pi(domain, target))
    }

    pub fn application(&'a mut self, function: TermRef<'a>, argument: TermRef<'a>) -> TermRef<'a> {
        self.arena.alloc(Term::Application(function, argument))
    }
}
