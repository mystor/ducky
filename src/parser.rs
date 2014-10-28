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

pub type Parser<'a, A> = Box<ParserAction<'a, A> + 'static>;

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

pub fn white<'a>() -> Parser<'a, ()> {
    parser!([] (st) -> () {
        let &State(s, l) = st;
        let ns = s.trim_left_chars(|c: char| c.is_whitespace());
        Ok(((), State(ns, l.advance(s.slice_to(s.len() - ns.len())))))
    })
}

pub fn charp<'a>(pred: fn (&char) -> bool) -> Parser<'a, char> {
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

pub fn some_char<'a>(patt: char) -> Parser<'a, char> {
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

pub fn string<'a, 'b>(patt: MaybeOwned<'b>) -> Box<ParserAction<'a, &'a str> + 'b> {
    struct Closure<'b>(MaybeOwned<'b>);
    impl <'a, 'b>ParserAction<'a, &'a str> for Closure<'b> {
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

// pub fn string<'a, 'b>(patt: &'b str) -> Box<ParserAction<'a, &'a str> + 'b> {
//     struct Closure<'b>(&'b str);
//     impl <'a, 'b>ParserAction<'a, &'a str> for Closure<'b> {
//         fn run(&self, st: &State<'a>) -> Result<(&'a str, State<'a>), String> {
//             let &Closure(patt,) = self;
//             let &State(s, l) = st;
//             let len = s.len();
//             if len < patt.len() { return Err("Unexpected End of Input".to_string()) }
            
//             let ss = s.slice_to(len);
//             if ss == patt {
//                 Ok((ss, State(s.slice_from(len), l.advance(ss))))
//             } else {
//                 Err("Unexpected Text".to_string())
//             }
//         }
//     }
//     box Closure(patt)
// }

// pub fn white<'a>() -> Parser<'a, ()> {
//     lambda!([] (st: &State<'a>) -> ParserResult<'a, ()> {
//         let &State(s, l) = st;
//         let ns = s.trim_left_chars(|c: char| c.is_whitespace());
//         Ok(((), State(ns, l.advance(s.slice_to(s.len() - ns.len())))))
//     })
// }


// pub fn whitespace<'a>(st: &State<'a>) -> Parsed<'a, ()> {
//     let &State(s, l) = st;
//     let ns = s.trim_left_chars(|c: char| c.is_whitespace());
//     Ok(((), State(ns, l.advance(s.slice_to(s.len() - ns.len())))))
// }
// 
// pub fn charp<'a>(pred: fn (&char) -> bool, st: &State<'a>) -> Parsed<'a, char> {
//     let &State(s, l) = st;
//     if s.len() == 0 { return Err("Unexpected End of Input".to_string()); }
// 
//     let c = s.char_at(0);
//     if pred(&c) {
//         Ok((c, State(s.slice_from(1), l.next(&c))))
//     } else {
//         Err(format!("Unexpected {}", c))
//     }
// }
// 
// pub fn string<'a>(patt: &str, st: &State<'a>) -> Parsed<'a, &'a str> {
//     let &State(s, l) = st;
//     let len = patt.len();
//     if s.len() < len { return Err(format!("Expected '{}', found '{}'", patt, s)); }
//     
//     let os = s.slice_to(len);
//     if os == patt {
//         Ok((os, State(s.slice_from(len), l.advance(os))))
//     } else {
//         Err(format!("Expected '{}', found '{}'", patt, os))
//     }
// }
// 
// pub fn wstring<'a>(patt: &str, st: &State<'a>) -> Parsed<'a, &'a str> {
//     let (_, st) = try!(whitespace(st));
//     string(patt, &st)
// }
// 
// pub fn regs<'a>(patt: Regex, st: &State<'a>) -> Parsed<'a, &'a str> {
//     let &State(s, l) = st;
//     if let Some((0, len)) = patt.find(s) {
//         let v = s.slice_to(len);
//         let ns = s.slice_from(len);
//         Ok((v, State(ns, l.advance(v))))
//     } else {
//         Err(format!("Unexpected"))
//     }
// }
// 
// pub fn wregs<'a>(patt: Regex, st: &State<'a>) -> Parsed<'a, &'a str> {
//     let (_, st) = try!(whitespace(st));
//     wregs(patt, &st)
// }
// 
// pub fn parseint<'a>(st: &State<'a>) -> Parsed<'a, &'a str> {
//     wregs(regex!(r"^[0-9]+"), st)
// }

pub fn literal<'a>(toks: &'a [Token]) -> Result<(Expr, &'a [Token]), String> {
    match toks[0] {
        LIT_INTEGER(x) => Ok((IntExpr(x), toks.slice_from(1))),
        LIT_FLOAT(x) => Ok((FloatExpr(x), toks.slice_from(1))),
        _ => Err(format!("Expected literal, found {}", toks[0]))
    }
}
