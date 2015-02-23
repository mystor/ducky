
// TODO(michael): Implement real tests for code generation (not this)
// (possibly including mcjit? Who knows!)

/*
use gen;
use infer;
use lexer;
use parser;
use scope;

/// Compiles some code, and then does stuff.
fn gen_code(code: &str) -> Result<(), String> {
    // TODO: The program should probably panic if the error wasn't caused by infer_program(ast)
    // Because these tests are only supposed to be testing inference, not lexing/parsing
    let tokens = try!(lexer::lex(code));
    let ast = try!(parser::parse_program(&mut parser::State::new(tokens.as_slice())));
    let scoped_ast = try!(scope::scoped_block(&mut scope::Scope::new(), ast.as_slice()));
    try!(infer::infer_program(scoped_ast.clone()));
    gen::gen_code(scoped_ast);
    Ok(())
}

/// Asserts that there was no error when typechecking the given code
fn gen_print(code: &str) {
    gen_code(code).unwrap();
}

#[test]
fn compose_identity() {
    gen_print(stringify!{
        let id = fn(x) { x };

        let id2 = id(id)(id);
        id(5);
        id2(5);
    });
}

#[test]
fn add_ints() {
    gen_print(stringify!{
        1 + 3;
    });
}

#[test]
fn mul_ints() {
    gen_print(stringify!{
        1 * 3;
    });
}

#[test]
fn infer_through_fn_call() {
    gen_print(stringify!{
        let id = fn(x) { x };
        let f = fn(y) {
            id(y).prop
        };
    });
}

#[test]
fn infer_args_through_intermediate_vars() {
    gen_print(stringify!{
        let id = fn(x) { x };
        let f = fn(y) {
            let z = id(y);
            z.prop
        };
        f({ prop: {} });
    });
}

#[test]
fn homogenous_if() {
    gen_print(stringify!{
        let f = fn(x) {
            if x {1} else {2}
        }
    });

    gen_print(stringify!{
        let f = fn(x) {
            if x { {a: 2} } else { {a: 3} }
        }
    });

    gen_print(stringify!{
        let f = fn(x) {
            if x { {a: 2, b: 2} } else { {a: 3, b: 2} }.a
        }
    });

    gen_print(stringify!{
        let f = fn(x) {
            if x {1} else {2} + 1
        }
    });
}

#[test]
fn non_homogenous_if() {
    // Can have a non-homogenous if statement
    gen_print(stringify!{
        let f = fn(x) {
            if x {
                { a: 2, b: 3 }
            } else {
                { a: 3 }
            }
        }
    });

    // Can access the a property (common between branches)
    gen_print(stringify!{
        let f = fn(x) {
            if x {
                { a: 2, b: 3 }
            } else {
                { a: 3 }
            }.a
        }
    });
}

#[test]
fn nested_non_homo_if() {
    gen_print(stringify!{
        let f = fn(x, y) {
            if x {
                if y {
                    { a: 3, b: 4 }
                } else {
                    { a: {}, b: 10 }
                }
            } else {
                { a: {}, c: 10 }
            }.a // We should be able to access the a property
        }
    });
}

#[test]
fn union_as_fn_arg() {
    gen_print(stringify!{
        // G produces a union when passed a bool
        let g = fn(y) {
            if y {
                { a: 5, b: 10 }
            } else {
                { a: {} }
            }
        };

        // H will try to use the a property of something
        let h = fn(z) {
            z.a
        };

        // F is our "entry" point as we can't create bools yet
        let f = fn(x) {
            h(g(x))
        };
    });
}
*/
