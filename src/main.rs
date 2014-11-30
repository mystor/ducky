#![feature(phase, if_let, macro_rules, globs)]

// We're going to use a lot of regular expressions
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

// Logging macros
#[phase(plugin, link)]
extern crate log;

// Interned Strings (from servo)
extern crate string_cache;

// LLVM bindings (from rustc)
extern crate rustc_llvm;

pub mod lexer;
pub mod parser;
pub mod il;
pub mod infer;
pub mod simplify;
pub mod gen;

fn infer_types_for_code(code: &str) {
    info!("-----------------------");
    info!("{}", code);
    info!("-----------------------");

    let tokens = lexer::lex(code);
    match tokens {
        Ok(rawtoks) => {
            let ast = parser::parse_program(&mut parser::State::new(rawtoks.as_slice()));
            info!("Tokens: {}", rawtoks);
            match ast {
                Ok(rawast) => {
                    info!("AST: {}", rawast);
                    let inferred_types = infer::infer_program(rawast);
                    match inferred_types {
                        Ok(ref types) => {
                            info!("Unsimplified: {}", inferred_types);
                            println!("{}", simplify::simplify(types));
                        }
                        Err(err) => {
                            println!("Error inferring types: {}", err);
                        }
                    }
                }
                Err(err) => {
                    println!("Error parsing: {}", err);
                }
            }
        }
        Err(err) => {
            println!("Error lexing: {}", err);
        }
    }
}

fn main() {
    // Records!
    infer_types_for_code(r#"
let magnitude = fn(pt) {
  sqrt(pt.x * pt.x + pt.y * pt.y)
};

let z = {x: 10, y: 20};

magnitude(z);
"#);

    // let polymorphism!
    infer_types_for_code(r#"
let id = fn(x) { x };

let y = id(id);
let z = id(5);
"#);

    // let polymorphism!
    infer_types_for_code(r#"
let x = 10;
let res = match x {
    { a: Int } as y => 1;
    Int as z => {
        1 + 1
    }
};
"#);
    gen::gen();
}
