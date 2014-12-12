use std::collections::HashMap;
use std::iter::{Counter, count};
use std::cell::RefCell;
use std::rc::Rc;
use il::*;

#[deriving(Clone)]
pub struct Scope {
    counter: Rc<RefCell<Counter<uint>>>,
    subs: HashMap<Ident, (Ident, int)>,
}

impl Scope {
    fn next(&self) -> uint {
        self.counter.borrow_mut().next().unwrap()
    }
}

macro_rules! builtin {
    ($subs:expr <- [$($ident:expr),+]) => {
        {
            $($subs.insert(
                ::il::Ident::from_slice($ident),
                (::il::Ident::from_builtin_slice($ident), 1));)+
        }
    }
}

impl Scope {
    pub fn new() -> Scope {
        let mut subs = HashMap::new();

        builtin!(subs <- [
            "true",
            "false",
            "null",
            "Int"
            ])

        Scope{
            counter: Rc::new(RefCell::new(count(0, 1))),
            subs: subs,
        }
    }
}

pub fn scoped_expr(scope: &mut Scope, expr: &Expr) -> Result<Expr, String> {
    match *expr {
        Expr::Literal(_) => Ok(expr.clone()),
        Expr::Ident(ref id) => {
            if let Some(&(ref id, ref mut use_count)) = scope.subs.get_mut(id) {
                *use_count += 1;
                Ok(Expr::Ident(id.clone()))
            } else {
                Err(format!("Use of undeclared variable: {}", id))
            }
        }
        Expr::Rec(ref props) => {
            Ok(Expr::Rec(try!(props.iter().map(|prop| {
                match *prop {
                    Prop::Val(ref symb, ref expr) => {
                        Ok(Prop::Val(symb.clone(), try!(scoped_expr(scope, expr))))
                    }
                    Prop::Method(ref symb, ref args, ref body) => {
                        let mut nscope = scope.clone();

                        // Bind all of the variables in args!
                        for arg in args.iter() {
                            let sub = arg.scoped_with_depth(nscope.next());
                            nscope.subs.insert(arg.clone(), (sub, 0));
                        }

                        Ok(Prop::Method(symb.clone(),
                                        args.clone(),
                                        try!(scoped_expr(&mut nscope, body))))
                    }
                }
            }).collect())))
        }

        Expr::Member(box ref expr, ref symb) => {
            Ok(Expr::Member(box try!(scoped_expr(scope, expr)), symb.clone()))
        }
        Expr::Call(box ref callee, ref symb, ref args) => {
            Ok(Expr::Call(
                box try!(scoped_expr(scope, callee)),
                symb.clone(),
                try!(args.iter().map(|x| scoped_expr(scope, x)).collect())))
        }

        Expr::Block(ref stmts) => {
            Ok(Expr::Block(try!(scoped_block(scope, stmts.as_slice()))))
        }
        Expr::If(box ref cond, box ref cons, box ref alt) => {
            Ok(Expr::If(
                box try!(scoped_expr(scope, cond)),
                box try!(scoped_expr(scope, cons)),
                box match *alt {
                    Some(ref x) => Some(try!(scoped_expr(scope, x))),
                    None => None
                }))
        }

    }
}

pub fn scoped_block(scope: &mut Scope, stmts: &[Stmt]) -> Result<Vec<Stmt>, String> {
    // Create a new scope
    let mut nscope = scope.clone();

    // Add the variables bound in this context
    for stmt in stmts.iter() {
        if let Stmt::Let(ref id, _) = *stmt {
            let sub = id.scoped_with_depth(nscope.next());
            nscope.subs.insert(id.clone(), (sub, 0));
        }
    }

    // Recur on statements
    stmts.iter().map(|stmt| {
        match *stmt {
            Stmt::Let(ref id, ref expr) => {
                // This is safe because we just created it!
                let nid = {
                    let &(ref nid, _) = nscope.subs.get(id).unwrap();
                    nid.clone()
                };

                let nexpr = try!(scoped_expr(&mut nscope, expr));
                Ok(Stmt::Let(nid, nexpr))
            }
            Stmt::Expr(ref expr) => {
                let nexpr = try!(scoped_expr(&mut nscope, expr));
                Ok(Stmt::Expr(nexpr))
            }
            Stmt::Empty => Ok(Stmt::Empty)
        }
    }).collect()
}


#[cfg(test)]
mod test {
    use il::*;
    use scope::*;
    use lexer;
    use parser;

    fn scope(code: &str) -> Result<Vec<Stmt>, String> {
        let toks = lexer::lex(code).unwrap();
        let ast = parser::parse_program(&mut parser::State::new(toks.as_slice())).unwrap();
        scoped_block(&mut Scope::new(), ast.as_slice())
    }

    fn scope_ok(code: &str) {
        match scope(code) {
            Ok(_) => {},
            Err(reason) => {
                panic!("\nUnexpected error scoping code:\n\n{}\n\n{}\n", code, reason);
            }
        }
    }

    fn scope_err(code: &str) {
        match scope(code) {
            Err(_) => {},
            Ok(stmts) => {
                panic!("\nUnexpected error scoping code:\n\n{}\n\n{}\n", code, stmts);
            }
        }
    }

    #[test]
    fn let_binds_var() {
        scope_ok(stringify!{
            let x = 10;
            x + 5;
        });

        scope_ok(stringify!{
            let x = 10;
            if true {
                x + 5
            } else {
                x - 5
            }
        });
    }

    #[test]
    fn let_constrained_by_block() {
        scope_err(stringify!{
            if true {
                let x = 20;
                x
            };
            x
        });

        scope_err(stringify!{
            if true {
                5
            } else {
                let x = 20;
                x
            };
            x
        });
    }

    #[test]
    fn function_binds_var() {
        scope_ok(stringify!{
            let f = fn(a) {
                a
            };
        });

        scope_ok(stringify!{
            let f = fn(a) {
                a
            };

            let g = fn(a) {
                a + a
            };
        });

        scope_err(stringify!{
            let f = fn(a) {
                a
            };

            a;
        });
    }

    #[test]
    fn reject_undefined_var() {
        scope_err(stringify!{
            let x = y;
        });

        scope_err(stringify!{
            let f = fn(x, y) {
                x + y
            };
            let z = x;
        });
    }
}
