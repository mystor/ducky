#![feature(phase, if_let, macro_rules)]

// We're going to use a lot of regular expressions
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

pub mod ast;
pub mod lexer;
pub mod parser;

fn main() {
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
}
