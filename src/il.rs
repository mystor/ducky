use intern::Atom;
use std::fmt;

// TODO: Namespace Context
pub use self::Context::*;

#[derive(Show, PartialEq, Eq, Hash, Clone)]
pub enum Context {
    Internal(u32),
    BuiltIn,
    User(u32),
    Unresolved, // Unresolved values have just been read in by the program
}

//| Identifiers are names used for type and data variables
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Ident(pub Atom, pub Context);

impl Ident {
    pub fn from_atom(atom: &Atom) -> Ident {
        Ident(atom.clone(), Unresolved)
    }

    pub fn from_slice(s: &str) -> Ident {
        Ident(Atom::from_slice(s), Unresolved)
    }

    pub fn from_builtin_slice(s: &str) -> Ident {
        Ident(Atom::from_slice(s), BuiltIn)
    }

    pub fn scoped_with_depth(&self, depth: u32) -> Ident {
        let Ident(ref atom, _) = *self;
        Ident(atom.clone(), User(depth))
    }
}

impl fmt::Show for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Ident(ref atom, ref context) = *self;
        match *context {
            User(i) => {
                write!(f, "{}~{}", i, atom.as_slice())
            }
            BuiltIn => {
                write!(f, "::{}", atom.as_slice())
            }
            Internal(i) => {
                if i < 52 {
                    write!(f, "_{}", atom.as_slice())
                } else {
                    write!(f, "{}::{}", i, atom.as_slice())
                }
            }
            Unresolved => {
                write!(f, "{}", atom.as_slice())
            }
        }
    }
}

//| Symbols are names used for properties and methods
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Symbol(pub Atom);

impl Symbol {
    pub fn from_atom(atom: &Atom) -> Symbol {
        Symbol(atom.clone())
    }

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

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TyProp {
    Val(Symbol, Ty),
    Method(Symbol, Vec<Ty>, Ty),
}

impl TyProp {
    pub fn symbol<'a>(&'a self) -> &'a Symbol {
        match *self {
            TyProp::Val(ref s, _) => s,
            TyProp::Method(ref s, _, _) => s,
        }
    }
}

impl fmt::Show for TyProp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TyProp::Val(ref symbol, ref ty) => {
                write!(f, "{:?}: {:?}", symbol, ty)
            }
            TyProp::Method(ref symbol, ref args, ref res) => {
                // @TODO: This is terrible syntax, but must differentiate
                // from ValTyProp
                try!(write!(f, "fn {:?}(", symbol));
                for arg in args.iter() {
                    try!(write!(f, "{:?}", arg));
                }
                write!(f, ") -> {:?}", res)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Ident(Ident),
    Rec(Option<Box<Ty>>, Vec<TyProp>),
    Union(Vec<Ty>),
}

impl Ty {
    pub fn unwrap_ident(&self) -> Ident {
        match *self {
            Ty::Ident(ref id) => id.clone(),
            _ => panic!("ICE: Couldn't Unwrap Identifier"),
        }
    }

    pub fn get_extends(&self) -> Option<&Ty> {
        match *self {
            Ty::Ident(_) => Some(self),
            Ty::Rec(Some(box ref ty), _) => Some(ty),
            _ => None,
        }
    }

    pub fn is_ident(&self) -> bool {
        if let Ty::Ident(..) = *self { true } else { false }
    }

    pub fn is_rec(&self) -> bool {
        if let Ty::Rec(..) = *self { true } else { false }
    }

    pub fn is_union(&self) -> bool {
        if let Ty::Union(..) = *self { true } else { false }
    }
}

impl fmt::Show for Ty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Ty::Ident(ref id) => write!(f, "{:?}", id),
            Ty::Rec(ref maybe_ty, ref props) => {
                if let Some(box ref ty) = *maybe_ty {
                    try!(write!(f, "{:?}:{:?} ", ty, "{"));
                    for prop in props.iter() {
                        try!(write!(f, "{:?}, ", prop));
                    }
                    write!(f, "{:?}", "}")
                } else {
                    try!(write!(f, "{:?} ", "{"));
                    for prop in props.iter() {
                        try!(write!(f, "{:?}, ", prop));
                    }
                    write!(f, "{:?}", "}")
                }
            }
            Ty::Union(ref options) => {
                try!(write!(f, "("));
                for option in options.iter() {
                    try!(write!(f, "{:?} |", option));
                }
                write!(f, ")")
            }
        }
    }
}


#[derive(Show, Clone)]
pub enum Literal {
    Str(Atom),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Literal {
    pub fn ty(&self) -> Ty {
        Ty::Ident(match *self {
            Literal::Str(_) => Ident(Atom::from_slice("Str"), BuiltIn),
            Literal::Int(_) => Ident(Atom::from_slice("Int"), BuiltIn),
            Literal::Float(_) => Ident(Atom::from_slice("Float"), BuiltIn),
            Literal::Bool(_) => Ident(Atom::from_slice("Bool"), BuiltIn),
        })
    }
}

#[derive(Show, Clone)]
pub enum Prop {
    Val(Symbol, Expr),
    Method(Symbol, Vec<Ident>, Expr),
}

#[derive(Show, Clone)]
pub enum Expr {
    Literal(Literal),
    Ident(Ident),
    Rec(Vec<Prop>),

    Member(Box<Expr>, Symbol),
    Call(Box<Expr>, Symbol, Vec<Expr>),

    Block(Vec<Stmt>),
    If(Box<Expr>, Box<Expr>, Box<Option<Expr>>),
}

#[derive(Show, Clone)]
pub enum Stmt {
    Let(Ident, Expr),
    Expr(Expr),
    Empty,
}
