#![feature(phase, if_let, macro_rules, globs)]

// We're going to use a lot of regular expressions
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

#[phase(plugin, link)]
extern crate log;

extern crate string_cache;

pub mod il;
pub mod infer;
pub mod ast;
pub mod lexer;
pub mod parser;

fn main() {
    let tokens = lexer::lex(r#"
let magnitude = fn(pt) {
  sqrt(pt.x * pt.x + pt.y * pt.y)
};

let z = {x: 10, y: 20};

magnitude(z);
"#);
    match tokens {
        Ok(rawtoks) => {
            println!("{}", rawtoks);
            println!("{}", parser::parse_program(&mut parser::State::new(rawtoks.as_slice())));
        }
        Err(err) => {
            println!("Error lexing: {}", err);
        }
    }
}
