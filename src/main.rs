#![feature(phase, if_let, macro_rules, globs)]

// We're going to use a lot of regular expressions
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

#[phase(plugin, link)]
extern crate log;

extern crate string_cache;

pub mod lexer;
pub mod parser;
pub mod il;
pub mod infer;
pub mod simplify;

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
}
