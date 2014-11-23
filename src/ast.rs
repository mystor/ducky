use string_cache::Atom;

// TODO: Consider merging ast and il

#[deriving(Show, Clone)]
pub struct Ident(pub Atom);

impl Ident {
    pub fn from_slice(s: &str) -> Ident {
        Ident(Atom::from_slice(s))
    }
}

#[deriving(Show, Clone)]
pub enum Literal {
    Str(Atom),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[deriving(Show, Clone)]
pub enum Prop {
    Val(Ident, Expr),
    Method(Ident, Vec<Ident>, Expr),
}

#[deriving(Show, Clone)]
pub enum Call {
    Fn(Box<Expr>, Vec<Expr>),
    Method(Box<Expr>, Ident, Vec<Expr>),
}

#[deriving(Show, Clone)]
pub enum Expr {
    Literal(Literal),
    Ident(Ident),
    Rec(Vec<Prop>),
    Member(Box<Expr>, Ident),
    Call(Call),
    Fn(Vec<Ident>, Box<Expr>),
    Block(Vec<Stmt>),
}

#[deriving(Show, Clone)]
pub enum Stmt {
    Let(Ident, Expr),
    Expr(Expr),
    Empty
}
