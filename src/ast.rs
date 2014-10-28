#[deriving(Show)]
pub struct Name(String);

#[deriving(Show)]
pub struct Attr(Name, Type);

#[deriving(Show)]
pub enum Type {
    ObjectTy(Vec<Attr>),
    IntTy,
    FloatTy,
    StringTy,
}

#[deriving(Show)]
pub enum Item {
    TypeItem(Name, Type),
    StmtItem(Stmt),
}

#[deriving(Show)]
pub enum Stmt {
    DeclStmt(Name, Expr),
    ExprStmt(Expr),
}

#[deriving(Show)]
pub enum Expr {
    IntExpr(int),
    StrExpr(String),
    FloatExpr(f64),
}
