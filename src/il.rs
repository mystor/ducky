use string_cache::Atom;

#[deriving(Show, PartialEq, Eq, Hash, Clone)]
pub enum Context {
    Internal(int),
    BuiltIn,
    User,
}

#[deriving(Show, PartialEq, Eq, Hash, Clone)]
pub struct Ident(pub Atom, pub Context);

#[deriving(Show, Clone)]
pub enum TyProp {
    ValTyProp(Ident, Ty),
    MethodTyProp(Ident, Ty),
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
    ValProp(Ident, Expr),
    MethodProp(Ident, Expr),
}

#[deriving(Show, Clone)]
pub enum Call {
    FnCall(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, Ident, Vec<Expr>),
}

#[deriving(Show, Clone)]
pub enum Expr {
    LiteralExpr(Literal),
    IdentExpr(Ident, Option<Ty>),
    RecExpr(Vec<Prop>, Option<Ty>),
    CallExpr(Call, Option<Ty>),
    FnExpr(Vec<Ident>, Vec<Stmt>, Option<Ty>),
}

#[deriving(Show, Clone)]
pub enum Stmt {
    LetStmt(Ident, Expr),
    ExprStmt(Expr),
}