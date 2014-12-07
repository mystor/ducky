use infer;
use lexer;
use parser;

/// Compiles some code, and then infers its type.
fn infer_code(code: &str) -> Result<infer::InferValue, String> {
    let tokens = try!(lexer::lex(code));
    let ast = try!(parser::parse_program(&mut parser::State::new(tokens.as_slice())));
    infer::infer_program(ast)
}

/// Asserts that there was an error when typechecking the given code
fn infer_err(code: &str) {
    match infer_code(code) {
        Ok(_) => {
            panic!("Unexpected success inferring types for code:\n\n{}", code);
        }
        Err(_) => {}
    }
}

/// Asserts that there was no error when typechecking the given code
fn infer_ok(code: &str) {
    match infer_code(code) {
        Err(e) => {
            panic!("Unexpected error inferring types for code:\n{}\n{}", e, code);
        }
        Ok(_) => {}
    }
}


#[test]
fn test_compose_identity() {
    infer_ok(stringify!{
        let id = fn(x) { x };

        let id2 = id(id)(id);
        id2(5);
    });
}

#[test]
fn test_add_ints() {
    infer_ok(stringify!{
        let add = fn (a, b) {
            a + b
        };
        add(1, 3);
    });
}

#[test]
fn test_mul_ints() {
    infer_ok(stringify!{
        let mul = fn (a, b) {
            a * b
        };
        mul(1, 3);
    });
}

#[test]
fn test_mul_random_records() {
    infer_err(stringify!{
        {a: 5, b: 20} * {c: 30};
    });
}
