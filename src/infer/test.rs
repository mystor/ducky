use infer;
use lexer;
use parser;

/// Compiles some code, and then infers its type.
fn infer_code(code: &str) -> Result<infer::InferValue, String> {
    // TODO: The program should probably panic if the error wasn't caused by infer_program(ast)
    // Because these tests are only supposed to be testing inference, not lexing/parsing
    let tokens = try!(lexer::lex(code));
    let ast = try!(parser::parse_program(&mut parser::State::new(tokens.as_slice())));
    infer::infer_program(ast)
}

/// Asserts that there was an error when typechecking the given code
fn infer_err(code: &str) {
    match infer_code(code) {
        Ok(istate) => {
            // TODO: Maybe simplify the types before displaying them?
            panic!("\nUnexpected success inferring types for code:\n\n{}\n\n{}\n", code, istate);
        }
        Err(_) => {}
    }
}

/// Asserts that there was no error when typechecking the given code
fn infer_ok(code: &str) {
    match infer_code(code) {
        Err(e) => {
            panic!("\nUnexpected error inferring types for code:\n\n{}\n\n{}\n", e, code);
        }
        Ok(_) => {}
    }
}

#[test]
fn compose_identity() {
    infer_ok(stringify!{
        let id = fn(x) { x };

        let id2 = id(id)(id);
        id(5);
        id2(5);
    });
}

#[test]
fn add_ints() {
    infer_ok(stringify!{
        1 + 3;
    });
}

#[test]
fn mul_ints() {
    infer_ok(stringify!{
        1 * 3;
    });
}

#[test]
fn mul_random_records() {
    infer_err(stringify!{
        {a: 5, b: 20} * {c: 30};
    });
}

#[test]
fn no_identity_transmute() {
    infer_err(stringify!{
        let id = fn (x) { x };

        let id2 = id(id);
        id2.foo;
    });
}

#[test]
fn infer_through_fn_call() {
    infer_ok(stringify!{
        let id = fn(x) { x };
        let f = fn(y) {
            id(y).prop
        };
    });
}

#[test]
fn infer_args_through_intermediate_vars() {
    infer_err(stringify!{
        let id = fn(x) { x };
        let f = fn(y) {
            let z = id(y);
            z.prop
        };
        f({});
    });

    infer_ok(stringify!{
        let id = fn(x) { x };
        let f = fn(y) {
            let z = id(y);
            z.prop
        };
        f({ prop: {} });
    });

    infer_err(stringify!{
        let f = fn(x) {
            let y = x;
            x.prop
        };
        f({});
    });
}

#[test]
fn homogenous_if() {
    infer_ok(stringify!{
        let f = fn(x) {
            if x {1} else {2}
        }
    });

    infer_ok(stringify!{
        let f = fn(x) {
            if x { {a: 2} } else { {a: 3} }
        }
    });

    infer_ok(stringify!{
        let f = fn(x) {
            if x { {a: 2, b: 2} } else { {a: 3, b: 2} }.a
        }
    });

    infer_err(stringify!{
        let f = fn(x) {
            if x { {a: 2} } else { {a: 3} } + 1
        }
    });

    infer_ok(stringify!{
        let f = fn(x) {
            if x {1} else {2} + 1
        }
    });
}

#[test]
fn non_homogenous_if() {
    // Can have a non-homogenous if statement
    infer_ok(stringify!{
        let f = fn(x) {
            if x {
                { a: 2, b: 3 }
            } else {
                { a: 3 }
            }
        }
    });

    // Can access the a property (common between branches)
    infer_ok(stringify!{
        let f = fn(x) {
            if x {
                { a: 2, b: 3 }
            } else {
                { a: 3 }
            }.a
        }
    });

    // Can't access the b property (only on one branch)
    infer_err(stringify!{
        let f = fn(x) {
            if x {
                { a: 2, b: 3 }
            } else {
                { a: 3 }
            }.b
        }
    });
}

#[test]
fn nested_non_homo_if() {
    infer_ok(stringify!{
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

    infer_err(stringify!{
        let f = fn(x, y) {
            if x {
                if y {
                    { a: 3, b: 4 }
                } else {
                    { a: {}, b: 10 }
                }
            } else {
                { a: {}, c: 10 }
            }.a + 5 // But we can't add to it, because it might be a non-integer!
        }
    });
}
