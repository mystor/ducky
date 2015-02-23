// TODO: Box syntax :'(
#![feature(plugin, box_syntax, box_patterns, slicing_syntax)]

// !!!!! TEMPORARY WARNING SILENCERS !!!!!
// TODO(michael): Show => Debug :(
#![feature(hash, core, std_misc, collections)]

// Compiler Plugins!
#![plugin(regex_macros)]
#![plugin(bindgen)]

// We're going to use a lot of regular expressions
// extern crate regex_macros;
extern crate regex;

#[macro_use]
extern crate lazy_static;

extern crate libc;

pub mod intern;
pub mod scope;
pub mod lexer;
pub mod parser;
pub mod il;
pub mod infer;
pub mod simplify;
pub mod gen;
pub mod specialize;

#[allow(dead_code)]
fn main() {
    // Do a quick test with some super simple code
    let code = r#"
5+5
"#;
    println!("Test");

    let tokens = lexer::lex(code).unwrap();
    let ast = parser::parse_program(&mut parser::State::new(tokens.as_slice())).unwrap();
    let scoped_ast = scope::scoped_block(&mut scope::Scope::new(), ast.as_slice()).unwrap();
    infer::infer_program(scoped_ast.clone()).unwrap();
    unsafe { gen::gen_code(scoped_ast); }
}
