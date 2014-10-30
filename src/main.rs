#![feature(phase, if_let, macro_rules)]

// We're going to use a lot of regular expressions
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

extern crate string_cache;

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod parserc;

fn main() {
    println!("{}", parserc::ab(&parserc::State("ac", parserc::Pos{line:1, col:0})));
    // Lex some input and show the tokens
    println!("{}", lexer::lex(r#"
type Obj = {};
type Point = {x: int, y: int};

fn magnitude(pt: Point) {
  return sqrt(float(pt.x * pt.x + pt.y * pt.y));
}

let x = 20;
let y = |x| { 20 };

let z = {x: 10, y: 20};

print(magnitude(z));
"#));
    if let Ok(otoks) = lexer::lex(r#"13"#) {
        println!("{}", parser::literal(otoks.as_slice()));
    }
}
