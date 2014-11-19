use string_cache::Atom;

#[deriving(Show)]
pub struct Ident(pub Atom);

#[deriving(Show)]
pub struct Attr(pub Ident, pub Type);

#[deriving(Show)]
pub enum Type {
    ObjectTy(Vec<Attr>),
    IntTy,
    FloatTy,
    StringTy,
}

#[deriving(Show)]
pub enum Item {
    TypeItem(Ident, Type),
    StmtItem(Stmt),
}

#[deriving(Show)]
pub enum Stmt {
    DeclStmt(Ident, Expr),
    ExprStmt(Expr),
}

#[deriving(Show)]
pub enum Expr {
    IdentExpr(Ident),
    IntExpr(int),
    StrExpr(String),
    FloatExpr(f64),
    FnExpr(Vec<Ident>, Box<Expr>),
    CallExpr(Box<Expr>, Box<Vec<Expr>>),
}
