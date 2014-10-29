use std::fmt;

use std::str::MaybeOwned;
use regex::Regex;
use lexer::{Token, LIT_INTEGER, LIT_FLOAT};
use ast::{Expr, IntExpr, FloatExpr};

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


#[deriving(Show, PartialEq, Eq, Clone)]
pub struct State<'a>(&'a str, Loc);

pub trait ParserAction<'a, A> {
    fn run(&self, st: &State<'a>) -> Result<(A, State<'a>), String>;
}

pub type Parser<'a, 'b, A> = Box<ParserAction<'a, A> + 'b>;

fn then<'cl1: 'cl, 'cl2: 'cl, 'cl, A: 'cl, B: 'cl>(p1: Parser<'cl, 'cl1, A>, p2: Parser<'cl, 'cl2, B>) -> Parser<'cl, 'cl, B> {
    struct Closure<'cl, 'cl1: 'cl, 'cl2: 'cl, A, B>(Parser<'cl, 'cl1, A>, Parser<'cl, 'cl2, B>);
    
    impl <'cl, 'cl1: 'cl, 'cl2: 'cl, A, B>ParserAction<'cl, B> for Closure<'cl, 'cl1, 'cl2, A, B> {
        fn run(&self, st: &State<'cl>) -> Result<(B, State<'cl>), String> {
            let &Closure(ref a1, ref a2) = self;
            let (_, st) = try!(a1.run(st));
            a2.run(&st)
        }
    }

    box Closure(p1, p2)
}




// fn then<'a: 'b, 'b, A: 'b, B: 'b>(p1: Parser<'a, 'b, A>, p2: Parser<'a, 'b, B>) -> Parser<'a, 'b, B> {
//     struct Closure<'a, 'b, A, B>(Parser<'a, 'b, A>, Parser<'a, 'b, B>);
// 
//     impl <'a, 'b, A, B>ParserAction<'a, B> for Closure<'a, 'b, A, B> {
//         fn run(&self, st: &State<'a>) -> Result<(B, State<'a>), String> {
//             let &Closure(ref a1, ref a2) = self;
//             let (_, st) = try!(a1.run(st));
//             a2.run(&st)
//         }
//     }
//     
//     box Closure(p1, p2)
// }
// 
// pub trait Thenable<'a, 'b> {
//     fn then<A>(self, other: Parser<'a, 'b, A>) -> Parser<'a, 'b, A>;
// }

// impl <'a, 'b, A>Thenable<'a, 'b> for Parser<'a, 'b, A> {
//     fn then<B>(self, other: Parser<'a, 'b, B>) -> Parser<'a, 'b, B> {
//         struct Closure<'a, 'b, A, B>(Parser<'a, 'b, A>, Parser<'a, 'b, B>);
//         impl <'a, 'b, A, B>ParserAction<'a, B> for Closure<'a, 'b, A, B> {
//             fn run(&self, st: &State<'a>) -> Result<(B, State<'a>), String> {
//                 let &Closure(ref a1, ref a2) = self;
//                 let (_, st) = try!(a1.run(st));
//                 a2.run(&st)
//             }
//         }
//         box Closure(self, other)
//     }
// }

macro_rules! parser {
    (
        [$($capt:ident : $cty:ty),+] ($st:ident) -> $ty:ty $body:expr
    ) => (
        {
            #[allow(dead_code)]
            struct Closure($($cty),*);
            impl <'a>ParserAction<'a, $ty> for Closure {
                fn run(&self, $st: &State<'a>) -> Result<($ty, State<'a>), String> {
                    let &Closure($($capt),*) = self; // Expand lambda captures
                    $body
                }
            }
            box Closure($($capt),*)
        }
    );
    (
        [] ($st:ident) -> $ty:ty $body:expr
    ) => (
        {
            #[allow(dead_code)]
            struct Closure;
            impl <'a>ParserAction<'a, $ty> for Closure {
                fn run(&self, $st: &State<'a>) -> Result<($ty, State<'a>), String> {
                    $body
                }
            }
            box Closure
        }
    )
}

pub fn white<'a>() -> Parser<'a, 'static, ()> {
    parser!([] (st) -> () {
        let &State(s, l) = st;
        let ns = s.trim_left_chars(|c: char| c.is_whitespace());
        Ok(((), State(ns, l.advance(s.slice_to(s.len() - ns.len())))))
    })
}

pub fn charp<'a>(pred: fn (&char) -> bool) -> Parser<'a, 'static, char> {
    parser!([pred: fn (&char) -> bool] (st) -> char {
        let &State(s, l) = st;
        if s.len() == 0 { return Err("Unexpected End of Input".to_string()) }

        let c = s.char_at(0);
        if pred(&c) {
            Ok((c, State(s.slice_from(1), l.next(&c))))
        } else {
            Err(format!("Unexpected {}", c))
        }
    })
}

pub fn some_char<'a>(patt: char) -> Parser<'a, 'static, char> {
    parser!([patt: char] (st) -> char {
        let &State(s, l) = st;
        if s.len() == 0 { return Err("Unexpected End of Input".to_string()) }

        let c = s.char_at(0);
        if c == patt {
            Ok((c, State(s.slice_from(1), l.next(&c))))
        } else {
            Err(format!("Unexpected {}", c))
        }
    })
}

pub fn string<'a, 'patt>(patt: MaybeOwned<'patt>) -> Parser<'a, 'patt, &'a str> {
    struct Closure<'patt>(MaybeOwned<'patt>);
    impl <'a, 'patt>ParserAction<'a, &'a str> for Closure<'patt> {
        fn run(&self, st: &State<'a>) -> Result<(&'a str, State<'a>), String> {
            let &Closure(ref patt,) = self;
            let &State(s, l) = st;
            let len = s.len();
            if len < patt.len() { return Err("Unexpected End of Input".to_string()) }
            
            let ss = s.slice_to(len);
            if ss == patt.as_slice() {
                Ok((ss, State(s.slice_from(len), l.advance(ss))))
            } else {
                Err("Unexpected Text".to_string())
            }
        }
    }
    box Closure(patt)
}

pub fn regex<'a>(patt: Regex) -> Parser<'a, 'static, &'a str> {
    struct Closure(Regex);
    impl <'a>ParserAction<'a, &'a str> for Closure {
        fn run(&self, st: &State<'a>) -> Result<(&'a str, State<'a>), String> {
            let &Closure(ref patt,) = self;
            let &State(s, l) = st;
            
            if let Some((0, len)) = patt.find(s) {
                let ns = s.slice_to(len);
                Ok((ns, State(s.slice_from(len), l.advance(ns))))
            } else {
                Err("Unexpected".to_string())
            }
        }
    }
    box Closure(patt)
}

pub fn sosososos<'a>(patt: MaybeOwned<'static>) -> Parser<'a, 'static, &'a str> {
    then(white(), string(patt))
}
// pub fn wstring<'a, 'patt>(patt: MaybeOwned<'patt>) -> Parser<'a, 'patt, &'a str> {
//     then(white(), string(patt))
// }

pub fn literal<'a>(toks: &'a [Token]) -> Result<(Expr, &'a [Token]), String> {
    match toks[0] {
        LIT_INTEGER(x) => Ok((IntExpr(x), toks.slice_from(1))),
        LIT_FLOAT(x) => Ok((FloatExpr(x), toks.slice_from(1))),
        _ => Err(format!("Expected literal, found {}", toks[0]))
    }
}
