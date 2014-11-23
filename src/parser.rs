use lexer::{Token};
use lexer::Token::*;
use ast::{Expr, Call, Prop, Ident, Literal, Stmt};

// TODO: Desugaring shouldn't happen inline!

macro_rules! expect {
    ($st:expr ~ $patt:pat) => {
        match $st.peek() {
            Some(& $patt) => { $st.eat(); },
            unexpected => { return Err(format!("Unexpected {}! {}", unexpected, line!())); }
        }
    };
    ($st:expr ~ $patt:pat => $expr:expr) => {
        match $st.peek() {
            Some(& $patt) => {
                $st.eat();
                $expr
            },
            unexpected  => { return Err(format!("Unexpected {}! {}", unexpected, line!())); }
        }
    }
}

pub struct State<'a> {
    tokens: &'a [Token],
    // TODO: Spans etc.
}

impl<'a> State<'a> {
    pub fn new(tokens: &'a [Token]) -> State<'a> {
        State{tokens: tokens}
    }

    fn peek(&self) -> Option<&'a Token> {
        self.tokens.head()
    }
    
    fn eat(&mut self) -> Option<&'a Token> {
        let tok = self.tokens.head();
        self.tokens = self.tokens.tail();
        tok
    }
}

pub fn parse_stmt<'a>(st: &mut State<'a>) -> Result<Stmt, String> {
    match st.peek() {
        Some(&LET) => { // let IDENT = EXPR
            st.eat();
            // Get the identifier
            expect!(st ~ IDENT(ref ident) => {
                expect!(st ~ EQ => {
                    let expr = try!(parse_expr(st));
                    Ok(Stmt::Let(Ident(ident.clone()), expr))
                })
            })
        },
        None | Some(&SEMI) => {
            Ok(Stmt::Empty)
        }
        _ => { // EXPR
            let expr = try!(parse_expr(st));
            Ok(Stmt::Expr(expr))
        }
    }
}

pub fn parse_expr<'a>(st: &mut State<'a>) -> Result<Expr, String> {
    // TODO: Add lower precidence operators! (like ==, >= etc)
    parse_pm(st)
}

/// Infix expressions are just method calls on the lhs argument
fn mk_infix(op: &str, lhs: Expr, rhs: Expr) -> Expr {
    Expr::Call(Call::Method(box lhs, Ident::from_slice(op), vec![rhs]))
}

/// Infix operators + and -
fn parse_pm<'a>(st: &mut State<'a>) -> Result<Expr, String> {
    let mut lhs = try!(parse_tdm(st));
    loop {
        match st.peek() {
            Some(&PLUS) => {
                st.eat();
                let rhs = try!(parse_tdm(st));
                lhs = mk_infix("+", lhs, rhs);
            }
            Some(&MINUS) => {
                st.eat();
                let rhs = try!(parse_tdm(st));
                lhs = mk_infix("-", lhs, rhs);
            }
            _ => break
        }
    }
    Ok(lhs)
}

fn parse_tdm<'a>(st: &mut State<'a>) -> Result<Expr, String> {
    let mut lhs = try!(parse_unary(st));
    loop {
        match st.peek() {
            Some(&STAR) => {
                st.eat();
                let rhs = try!(parse_unary(st));
                lhs = mk_infix("*", lhs, rhs);
            }
            Some(&SLASH) => {
                st.eat();
                let rhs = try!(parse_unary(st));
                lhs = mk_infix("/", lhs, rhs);
            }
            Some(&PERCENT) => {
                st.eat();
                let rhs = try!(parse_unary(st));
                lhs = mk_infix("%", lhs, rhs);
            }
            _ => break
        }
    }
    Ok(lhs)
}

/// Unary prefix operators
fn parse_unary<'a>(st: &mut State<'a>) -> Result<Expr, String> {
    fn mk_unary(op: &str, arg: Expr) -> Expr {
        Expr::Call(Call::Method(box arg, Ident::from_slice(op), vec![])) 
    }

    match st.peek() {
        Some(&NOT) => {
            st.eat();
            let rhs = try!(parse_deref(st));
            Ok(mk_unary("not", rhs))
        }
        Some(&MINUS) => {
            st.eat();
            let rhs = try!(parse_deref(st));
            Ok(mk_unary("negate", rhs))
        }
        _ => parse_deref(st)
    }
}

/// Dereferences, Array accesses, and function calls!
fn parse_deref<'a>(st: &mut State<'a>) -> Result<Expr, String> {
    let mut lhs = try!(parse_value(st));
    loop {
        match st.peek() {
            Some(&DOT) => {
                st.eat();
                expect!(st ~ IDENT(ref ident) => {
                    lhs = Expr::Member(box lhs, Ident(ident.clone()));
                })
            }
            Some(&LPAREN) => {
                st.eat();
                let args = try!(parse_args(st));
                expect!(st ~ RPAREN);
                lhs = Expr::Call(Call::Fn(box lhs, args));
            }
            Some(&LBRACKET) => {
                // TODO: Implement arrays n' shit
                unimplemented!()
            }
            Some(&LBRACE) => {
                // TODO: Implement object update
                unimplemented!()
            }
            _ => break
        }
    }
    Ok(lhs)
}

fn parse_args<'a>(st: &mut State<'a>) -> Result<Vec<Expr>, String> {
    let mut args = vec![];
    loop {
        // Check if we should finish here
        if let Some(&LPAREN) = st.peek() { break }

        let arg = try!(parse_expr(st));
        args.push(arg);
        
        // Check if we might have another argument
        // This allows for trailing commas in function calls.
        match st.peek() {
            Some(&COMMA) => st.eat(),
            _ => break
        };
    }
    Ok(args)
}

fn parse_params<'a>(st: &mut State<'a>) -> Result<Vec<Ident>, String> {
    let mut params = vec![];
    loop {
        if let Some(&IDENT(ref ident)) = st.peek() {
            st.eat();
            params.push(Ident(ident.clone()));
            match st.peek() {
                Some(&COMMA) => st.eat(),
                _ => break
            };
        } else {
            break
        }
    }
    Ok(params)
}

fn parse_value<'a>(st: &mut State<'a>) -> Result<Expr, String> {
    // TODO: Add if and match
    match st.peek() {
        Some(&FN) => { // Function Literal
            st.eat();
            expect!(st ~ LPAREN);
            let params = try!(parse_params(st));
            expect!(st ~ RPAREN);
            let body = try!(parse_block_expr(st));

            Ok(Expr::Fn(params, box body))
        }
        Some(&LBRACE) => { // Object Literal
            st.eat();
            let props = try!(parse_props(st));
            expect!(st ~ RBRACE);

            Ok(Expr::Rec(props))
        }
        // Trivial Cases
        Some(&IDENT(ref ident)) => {
            st.eat();
            Ok(Expr::Ident(Ident(ident.clone())))
        }
        Some(&LIT_INTEGER(i)) => {
            st.eat();
            Ok(Expr::Literal(Literal::Int(i)))
        }
        Some(&LIT_FLOAT(f)) => {
            st.eat();
            Ok(Expr::Literal(Literal::Float(f)))
        }
        Some(&LIT_STR(ref string)) => {
            st.eat();
            Ok(Expr::Literal(Literal::Str(string.clone())))
        }
        Some(&TRUE) => {
            st.eat();
            Ok(Expr::Literal(Literal::Bool(true)))
        }
        Some(&FALSE) => {
            st.eat();
            Ok(Expr::Literal(Literal::Bool(false)))
        }

        unexpected => Err(format!("Unexpected {}!", unexpected)),
    }
}

fn parse_block_expr<'a>(st: &mut State<'a>) -> Result<Expr, String> {
    expect!(st ~ LBRACE);
    let stmts = try!(parse_stmts(st));
    expect!(st ~ RBRACE);
    Ok(Expr::Block(stmts))
}

fn parse_props<'a>(st: &mut State<'a>) -> Result<Vec<Prop>, String> {
    let mut props = vec![];
    loop {
        match st.peek() {
            Some(&FN) => {
                st.eat();
                expect!(st ~ IDENT(ref ident) => {
                    expect!(st ~ LPAREN);
                    let params = try!(parse_params(st));
                    expect!(st ~ RPAREN);
                    let body = try!(parse_block_expr(st));

                    props.push(Prop::Method(Ident(ident.clone()), params, body));
                })
            },
            Some(&IDENT(ref ident)) => {
                st.eat();
                expect!(st ~ COLON);
                let value = try!(parse_expr(st));

                props.push(Prop::Val(Ident(ident.clone()), value));
            }
            _ => break
        };
        
        match st.peek() {
            Some(&COMMA) => st.eat(),
            _ => break
        };
    }
    Ok(props)
}

fn parse_stmts<'a>(st: &mut State<'a>) -> Result<Vec<Stmt>, String> {
    let mut stmts = vec![];
    loop {
        stmts.push(try!(parse_stmt(st)));
        match st.peek() {
            Some(&SEMI) => st.eat(),
            _ => break
        };
    }
    Ok(stmts)
}

pub fn parse_program<'a>(st: &mut State<'a>) -> Result<Vec<Stmt>, String> {
    // Right now programs are just lists of statements
    parse_stmts(st)
}

