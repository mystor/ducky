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

    pub fn from_builtin_slice(s: &str) -> Ident {
        Ident(Atom::from_slice(s), BuiltIn)
    }
}

impl fmt::Show for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Ident(ref atom, ref context) = *self;
        match *context {
            User => {
                write!(f, "{}", atom.as_slice())
            }
            BuiltIn => {
                write!(f, "::{}", atom.as_slice())
            }
            Internal(i) => {
                write!(f, "{}::{}", i, atom.as_slice())
            }
        }
    }
}

//| Symbols are names used for properties and methods
#[deriving(PartialEq, Eq, Hash, Clone)]
pub struct Symbol(pub Atom);

impl Symbol {
    pub fn from_slice(s: &str) -> Symbol {
        Symbol(Atom::from_slice(s))
    }
}

impl fmt::Show for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Symbol(ref atom) = *self;
        write!(f, "{}", atom.as_slice())
    }
}

#[deriving(Clone)]
pub enum TyProp {
    ValTyProp(Symbol, Ty),
    MethodTyProp(Symbol, Vec<Ty>, Ty),
}

impl TyProp {
    pub fn symbol<'a>(&'a self) -> &'a Symbol {
        match *self {
            ValTyProp(ref s, _) => s,
            MethodTyProp(ref s, _, _) => s,
        }
    }
}

impl fmt::Show for TyProp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValTyProp(ref symbol, ref ty) => {
                write!(f, "{}: {}", symbol, ty)
            }
            MethodTyProp(ref symbol, ref args, ref res) => {
                // @TODO: This is terrible syntax, but must differentiate
                // from ValTyProp
                try!(write!(f, "{}(", symbol));
                for arg in args.iter() {
                    try!(write!(f, "{}", arg));
                }
                write!(f, ") -> {}", res)
            }
        }
    }
}

#[deriving(Clone)]
pub enum Ty {
    IdentTy(Ident),
    RecTy(Box<Option<Ty>>, Vec<TyProp>),
    FnTy(Vec<Ty>, Box<Ty>),
}

impl Ty {
    pub fn unwrap_ident(&self) -> Ident {
        match *self {
            IdentTy(ref id) => id.clone(),
            _ => panic!("ICE: Couldn't Unwrap Identifier"),
        }
    }
}

impl fmt::Show for Ty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IdentTy(ref id) => write!(f, "{}", id),
            RecTy(box ref maybe_ty, ref props) => {
                if let Some(ref ty) = *maybe_ty {
                    try!(write!(f, "{}:{} ", ty, "{"));
                    for prop in props.iter() {
                        try!(write!(f, "{}", prop));
                    }
                    write!(f, " {}", "}")
                } else {
                    try!(write!(f, "{} ", "{"));
                    for prop in props.iter() {
                        try!(write!(f, "{}", prop));
                    }
                    write!(f, " {}", "}")
                }
            }
            FnTy(ref args, box ref res) => {
                try!(write!(f, "("));
                for arg in args.iter() {
                    try!(write!(f, "{}", arg));
                }
                write!(f, ") -> {}", res)
            }
        }
    }
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
    MethodProp(Symbol, Vec<Ident>, Expr),
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
