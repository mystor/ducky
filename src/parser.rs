use lexer::{Token};
use lexer::Token::*;
use il::{Expr, Call, Prop, Ident, Symbol, Literal, Stmt, Ty, TyProp};

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

/// The State struct contains the current parsing state for the parser
/// It acts as a source of tokens for the parser to use.
///
/// TODO: This should probably be in a tokens module/the lexer module
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
                    Ok(Stmt::Let(Ident::from_atom(ident), expr))
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
    Expr::Call(Call::Method(box lhs, Symbol::from_slice(op), vec![rhs]))
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
        Expr::Call(Call::Method(box arg, Symbol::from_slice(op), vec![]))
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
                    lhs = Expr::Member(box lhs, Symbol::from_atom(ident));
                })
            }
            Some(&COLON) => {
                st.eat();
                expect!(st ~ IDENT(ref ident) => {
                    expect!(st ~ LPAREN);
                    let args = try!(parse_args(st));
                    expect!(st ~ RPAREN);
                    lhs = Expr::Call(Call::Method(box lhs, Symbol::from_atom(ident), args));
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
            /// @TODO: Reassess, conflicts with `match x {}` and `if x {}`
            // Some(&LBRACE) => {
            //     // TODO: Implement object update
            //     unimplemented!()
            // }
            _ => break
        }
    }
    Ok(lhs)
}

fn parse_args<'a>(st: &mut State<'a>) -> Result<Vec<Expr>, String> {
    let mut args = vec![];
    loop {
        // Check if we should finish here
        if let Some(&RPAREN) = st.peek() { break }

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
            params.push(Ident::from_atom(ident));
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
        Some(&IF) => {
            st.eat();
            let cond = try!(parse_expr(st));
            let then = try!(parse_block_expr(st));
            let els = if let Some(&ELSE) = st.peek() {
                st.eat();
                Some(try!(parse_block_expr(st)))
            } else { None };

            Ok(Expr::If(box cond, box then, box els))
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
            Ok(Expr::Ident(Ident::from_atom(ident)))
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

                    props.push(Prop::Method(Symbol::from_atom(ident), params, body));
                })
            },
            Some(&IDENT(ref ident)) => {
                st.eat();
                expect!(st ~ COLON);
                let value = try!(parse_expr(st));

                props.push(Prop::Val(Symbol::from_atom(ident), value));
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

fn parse_ty<'a>(st: &mut State<'a>) -> Result<Ty, String> {
    match st.peek() {
        Some(&FN) => {
            st.eat();
            // Function Type
            expect!(st ~ LBRACKET);
            let param_tys = try!(parse_paramtys(st));
            expect!(st ~ RBRACKET);
            expect!(st ~ RARROW);
            let result_type = try!(parse_ty(st));

            Ok(Ty::Fn(param_tys, box result_type))
        }
        Some(&LBRACE) => {
            st.eat();
            // Record Type
            let props = try!(parse_proptys(st));
            expect!(st ~ RBRACE);

            Ok(Ty::Rec(box None, props))
        }
        Some(&IDENT(ref id)) => {
            st.eat();
            let ident_ty = Ty::Ident(Ident::from_atom(id));
            // Identifier type or extended record
            match st.peek() {
                Some(&COLON) => {
                    // Extended Record!
                    // Parse the base record, and then extend it
                    st.eat();
                    let record = try!(parse_ty(st));
                    if let Ty::Rec(box None, props) = record {
                        Ok(Ty::Rec(box Some(ident_ty), props))
                    } else {
                        Err(format!("Unexpected non-record type!"))
                    }
                }
                _ => {
                    // Its just an identifier
                    Ok(ident_ty)
                }
            }
        }
        unexpected => Err(format!("Unexpected {}!", unexpected)),
    }
}

fn parse_paramtys<'a>(st: &mut State<'a>) -> Result<Vec<Ty>, String> {
    let mut paramtys = vec![];

    loop {
        // Check if we should finish here
        if let Some(&RPAREN) = st.peek() { break }

        let paramty = try!(parse_ty(st));
        paramtys.push(paramty);

        // Check if we might have another argument
        // This allows for trailing commas in function calls.
        match st.peek() {
            Some(&COMMA) => st.eat(),
            _ => break
        };
    }

    Ok(paramtys)
}

fn parse_proptys<'a>(st: &mut State<'a>) -> Result<Vec<TyProp>, String> {
    let mut props = vec![];

    loop {
        match st.peek() {
            Some(&FN) => {
                st.eat();
                expect!(st ~ IDENT(ref ident) => {
                    expect!(st ~ LPAREN);
                    let params = try!(parse_paramtys(st));
                    expect!(st ~ RPAREN);
                    let body = try!(parse_ty(st));

                    props.push(TyProp::Method(Symbol::from_atom(ident), params, body));
                })
            },
            Some(&IDENT(ref ident)) => {
                st.eat();
                expect!(st ~ COLON);
                let value = try!(parse_ty(st));

                props.push(TyProp::Val(Symbol::from_atom(ident), value));
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

pub fn parse_program<'a>(st: &mut State<'a>) -> Result<Vec<Stmt>, String> {
    // Right now programs are just lists of statements
    parse_stmts(st)
}
