#[deriving(Show)]
pub struct Ident(String);
impl Ident {
    pub fn new(s: String) -> Ident {
        Ident(s)
    }
}

#[deriving(Show)]
pub struct Attr(Ident, Type);

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
    IntExpr(int),
    StrExpr(String),
    FloatExpr(f64),
    FnExpr(Vec<Ident>, Box<Expr>),
}
