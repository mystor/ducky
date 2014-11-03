use string_cache::Atom;

#[deriving(Show, PartialEq, Eq, Hash, Clone)]
pub enum Context {
    BuiltIn,
    User,
}

#[deriving(Show, PartialEq, Eq, Hash, Clone)]
pub struct Ident(pub Atom, pub Context);

#[deriving(Show, Clone)]
pub enum TyAttr {
    ValTyAttr(Ident, Ty),
    MethodTyAttr(Ident, Ty),
}

#[deriving(Show, Clone)]
pub enum Ty {
    ForAllTy(Ident, Box<Ty>),
    IdentTy(Ident),
    ObjTy(Vec<TyAttr>),
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
pub enum Attr {
    ValAttr(Ident, Expr),
    MethodAttr(Ident, Expr),
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
    ObjExpr(Vec<Attr>, Option<Ty>),
    CallExpr(Call, Option<Ty>),
    FnExpr(Vec<Ident>, Vec<Stmt>, Option<Ty>),
}

#[deriving(Show, Clone)]
pub enum Stmt {
    LetStmt(Ident, Expr),
    ExprStmt(Expr),
}
