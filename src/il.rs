use string_cache::Atom;
use std::fmt;

#[deriving(Show, PartialEq, Eq, Hash, Clone)]
pub enum Context {
    Internal(uint),
    BuiltIn,
    User,
}

//| Identifiers are names used for type and data variables
#[deriving(PartialEq, Eq, Hash, Clone)]
pub struct Ident(pub Atom, pub Context);

impl Ident {
    pub fn from_user_slice(s: &str) -> Ident {
        Ident(Atom::from_slice(s), User)
    }
}

impl fmt::Show for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Ident(ref atom, ref context) = *self;
        write!(f, "{}::{}", context, atom.as_slice())
    }
}

//| Symbols are names used for properties and methods
#[deriving(Show, PartialEq, Eq, Hash, Clone)]
pub struct Symbol(pub Atom);

impl Symbol {
    pub fn from_slice(s: &str) -> Symbol {
        Symbol(Atom::from_slice(s))
    }
}

#[deriving(Show, Clone)]
pub enum TyProp {
    ValTyProp(Symbol, Ty),
    MethodTyProp(Symbol, Ty),
}

#[deriving(Show, Clone)]
pub enum Ty {
    IdentTy(Ident),
    RecTy(Vec<TyProp>),
    FnTy(Vec<Ty>, Box<Ty>),
}

#[deriving(Show, Clone)]
pub enum Literal {
    StrLit(Atom),
    IntLit(i64),
    FloatLit(f64),
}

impl Literal {
    pub fn ty(&self) -> Ty {
        IdentTy(match *self {
            StrLit(_) => Ident(Atom::from_slice("Str"), BuiltIn),
            IntLit(_) => Ident(Atom::from_slice("Int"), BuiltIn),
            FloatLit(_) => Ident(Atom::from_slice("Float"), BuiltIn),
        })
    }
}

#[deriving(Show, Clone)]
pub enum Prop {
    ValProp(Symbol, Expr),
    MethodProp(Symbol, Expr),
}

#[deriving(Show, Clone)]
pub enum Call {
    FnCall(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, Symbol, Vec<Expr>),
}

#[deriving(Show, Clone)]
pub enum Expr {
    LiteralExpr(Literal),
    IdentExpr(Ident),
    RecExpr(Vec<Prop>),
    CallExpr(Call),
    FnExpr(Vec<Ident>, Box<Expr>),
    BlockExpr(Vec<Stmt>),
}

#[deriving(Show, Clone)]
pub enum Stmt {
    LetStmt(Ident, Expr),
    ExprStmt(Expr),
}
