use std::fmt;

use lexer::{Token};
use lexer::Token::{IDENT, LIT_INTEGER, LIT_FLOAT, LPAREN, RPAREN, COMMA};
use ast::{Expr, Ident};
use ast::Expr::{FnExpr, IntExpr, FloatExpr, IdentExpr};

#[deriving(PartialEq, Eq, Clone)]
pub struct Loc {
    line: int,
    col: int,
}

impl Loc {
    pub fn start() -> Loc {
        Loc{ line: 1, col: 0 }
    }

    pub fn next(&self, c: &char) -> Loc {
        match *c {
            '\n' => Loc{ line: self.line + 1, col: 0 },
            _ => Loc{ line: self.line, col: self.col + 1 },
        }
    }

    pub fn advance(&self, s: &str) -> Loc {
        let mut line = self.line;
        let mut col = self.col;

        for c in s.chars() {
            match c {
                '\n' => { line += 1; col = 0 },
                _ => { col += 1 }
            }
        }
        
        Loc { line: line, col: col }
    }
}

impl fmt::Show for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(l:{}, c:{})", self.line, self.col)
    }
}

macro_rules! token {
    ($tok:pat, $st:expr) => {
        tokenp(|t: &Token| {
            match *t {
                $tok => true,
                _ => false,
            }
        }, $st)
    }
}

macro_rules! alts {
    ($st:expr => $first:expr; :err $err: expr) => {
        match ($first)($st) {
            Ok(x) => Ok(x),
            Err(e) => {
                let mut errs = Vec::new();
                errs.push(e);
                ($err)(&mut errs)
            }
        }
    };
    ($st:expr => $first:expr, $($rest:expr),+; :err $err:expr) => {
        match ($first)($st) {
            Ok(x) => Ok(x),
            Err(e) => {
                alts!($st => $($rest),+; :err |errs: &mut Vec<String>| {
                    errs.push(e.clone());
                    ($err)(errs)
                })
            }
        }
    }
}

pub type State<'a> = &'a [Token];
pub type Parsed<'a, A> = Result<(A, State<'a>), String>;

pub fn tokenp<'a>(pred: |&Token| -> bool, st: State<'a>) -> Parsed<'a, &'a Token> {
    match st.head() {
        Some(tok) => {
            if pred(tok) {
                Ok((tok, st.tail()))
            } else {
                Err(format!("Unexpected {}", tok))
            }
        }
        None => Err("Unexpected End of File".to_string())
    }
}

pub fn parse_ident<'a>(st: State<'a>) -> Parsed<'a, Ident> {
    match token!(IDENT(_), st) {
        Ok((&IDENT(ref name), st)) => Ok((Ident(name.clone()), st)),
        Err(s) => Err(format!("{} Expected IDENT", s)),
        _ => unreachable!()
    }
}

pub fn parse_expr<'a>(st: State<'a>) -> Parsed<'a, Expr> {
    alts!(st =>
        |st| { parse_ident(st).map(|(i, st)| { (IdentExpr(i), st) }) },
        |st| { parse_fn(st) };

        :err |errs: &mut Vec<String>| {
            Err(format!("{}", errs))
        }
    )
}

pub fn parse_args<'a>(st: State<'a>) -> Parsed<'a, Vec<Ident>> {
    let mut args: Vec<Ident> = vec![];
    let mut currst = st;
    loop {
        if let Ok((arg, st)) = parse_ident(currst) {
            args.push(arg);
            
            match tokenp(|c: &Token| { match *c { COMMA => true, _ => false } }, st) {
                Ok((_, st)) => { currst = st },
                Err(_) => { return Ok((args, st)); }
            }
        } else {
            return Ok((args, currst));
        }
    }
}

pub fn parse_fn<'a>(st: State<'a>) -> Parsed<'a, Expr> {
    // fn (ARGS) EXPR
    // let (_, st) = try_token!(IDENT(String("fn")), st);
    let (_, st) = try!(token!(LPAREN, st));
    let (args, st) = try!(parse_args(st));
    let (_, st) = try!(token!(RPAREN, st));
    let (body, st) = try!(parse_expr(st));
    Ok((FnExpr(args, box body), st))
}
    
pub fn literal<'a>(toks: &'a [Token]) -> Result<(Expr, &'a [Token]), String> {
    match toks[0] {
        LIT_INTEGER(x) => Ok((IntExpr(x), toks.slice_from(1))),
        LIT_FLOAT(x) => Ok((FloatExpr(x), toks.slice_from(1))),
        _ => Err(format!("Expected literal, found {}", toks[0]))
    }
}
