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

pub struct Builder {
    arena: blink_alloc::Blink,
}

impl Builder {
    pub fn variable<'a>(&'a self, index: Index) -> TermRef<'a> {
        self.arena.put_no_drop(Term::Variable(index))
    }

    pub fn literal<'a>(&'a self, literal: Literal) -> TermRef<'a> {
        self.arena.put_no_drop(Term::Literal(literal))
    }

    pub fn lambda<'a>(&'a self, type_: TypeRef<'a>, body: TermRef<'a>) -> TermRef<'a> {
        self.arena.put_no_drop(Term::Lambda(type_, body))
    }

    pub fn pi<'a>(&'a self, domain: TypeRef<'a>, target: TermRef<'a>) -> TermRef<'a> {
        self.arena.put_no_drop(Term::Pi(domain, target))
    }

    pub fn application<'a>(
        &'a mut self,
        function: TermRef<'a>,
        argument: TermRef<'a>,
    ) -> TermRef<'a> {
        self.arena
            .put_no_drop(Term::Application(function, argument))
    }
}
