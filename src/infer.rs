use std::collections::HashMap;
use il::*;

struct Environment {
    // Variables in scope and their types
    vars: HashMap<Ident, Ty>,
    // 
}

impl Environment {
    fn insert_type(&mut self, id: Ident, ty: Ty) {
        self.vars.insert(id, ty);
    }

    fn lookup_type(&self, id: &Ident) -> Option<&Ty> {
        self.vars.find(id)
    }
}

pub fn type_check_prgm(stmts: &mut [Stmt]) -> Result<(), ()> {
    let mut env = Environment{
        vars: HashMap::new(),
    };

    type_check_stmts(stmts, &mut env)
}

fn type_check_stmts(stmts: &mut [Stmt], env: &mut Environment) -> Result<(), ()> {
    for stmt in stmts.iter_mut() {
        match *stmt {
            LetStmt(ref mut ident, ref mut expr) => {
                let ty = try!(type_check_expr(expr, env));
                env.insert_type(ident.clone(), ty);
            },
            ExprStmt(ref mut expr) => {
                try!(type_check_expr(expr, env));
            }
        }
    }
    Ok(())
}

fn type_check_expr(expr: &mut Expr, env: &mut Environment) -> Result<Ty, ()> {
    match *expr {
        LiteralExpr(ref lit) => { Ok(lit.ty()) },
        IdentExpr(ref id, ref mut ty) => {
            let nty = env.lookup_type(id);
            *ty = nty.map(|x| x.clone());
            nty.map(|x: &Ty| x.clone()).ok_or(())
        },
        _ => unimplemented!()
    }
}
