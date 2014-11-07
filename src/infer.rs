use string_cache::Atom;
use std::collections::HashMap;
use il::*;

#[deriving(Show)]
pub struct Environment {
    data_vars: HashMap<Ident, Ty>,
    type_vars: HashMap<Ident, Ty>,
    counter: int,
}

impl Environment {
    // Accessors for the data from the environment
    fn lookup_type_var(&self, id: &Ident) -> Option<Ty> {
        self.type_vars.find(id).map(|x| { x.clone() })
    }

    fn lookup_data_var(&self, id: &Ident) -> Option<Ty> {
        self.data_vars.find(id).map(|x| { x.clone() })
    }
    
    // Creating a unique type variable
    fn introduce_type_var(&mut self) -> Ident {
        self.counter += 1;
        // TODO: Better symbol name
        Ident(Atom::from_slice("A"), Internal(self.counter))
    }
    
    // Perform a substitution (bind the type variable id)
    fn substitute(&mut self, id: Ident, ty: Ty) {
        self.type_vars.insert(id, ty);
    }
}

pub fn unify(env: &mut Environment, a: &Ty, b: &Ty) -> Result<(), String> {
    // Generate a set of substitutions such that a == b in env
    match (a, b) {
        (&IdentTy(ref a), b) => {
            if let Some(ref ty) = env.lookup_type_var(a).clone() {
                // The type name is explicit, resolve it
                unify(env, ty, b)
            } else {
                // The type name is unbounded, substitute it for b
                env.substitute(a.clone(), b.clone());
                Ok(())
            }
        }
        (a, &IdentTy(ref b)) => {
            // TODO: Check if I should do this
            if let Some(ref ty) = env.lookup_type_var(b).clone() {
                unify(env, ty, a)
            } else {
                env.substitute(b.clone(), a.clone());
                Ok(())
            }
        }
        (&FnTy(ref aargs, ref ares), &FnTy(ref bargs, ref bres)) => {
            // Argument lists must have the same length for functions to unify
            // This is usually handled by currying which may exist in this language later
            if aargs.len() != bargs.len() {
                return Err(format!("Cannot unify {} and {}", a, b));
            }
            
            // Unify each of the arguments
            for (aarg, barg) in aargs.iter().zip(bargs.iter()) {
                try!(unify(env, aarg, barg));
            }
            
            // Unify the results
            unify(env, &**ares, &**bres)
        }
        (&RecTy(_), &RecTy(_)) => { unimplemented!() }
        _ => {
            Err(format!("Cannot unify {} and {}", a, b))
        }
    }
}

pub fn infer_expr(env: &mut Environment, e: &Expr) -> Result<Ty, String> {
    match *e {
        LiteralExpr(ref lit) => { Ok(lit.ty()) } // We probably can just inline that
        IdentExpr(ref ident) => {
            env.lookup_data_var(ident).ok_or(
                format!("ICE: Unable to lookup type variable for ident: {}", ident))
        }
        CallExpr(FnCall(ref callee, ref params)) => {
            let callee_ty = try!(infer_expr(env, &**callee));
            let mut param_tys = Vec::with_capacity(params.len());
            for param in params.iter() {
                match infer_expr(env, param) {
                    Ok(ty) => { param_tys.push(ty); }
                    Err(err) => { return Err(err); }
                }
            }
            let beta = IdentTy(env.introduce_type_var());
            // TODO: Vastly improve this error message
            try!(unify(env, &callee_ty, &FnTy(param_tys, box beta.clone())));
            Ok(beta)
        }
        CallExpr(_) => { unimplemented!() }
        FnExpr(ref params, ref body) => {
            let body_ty = try!(infer_expr(env, &**body));
            let mut param_tys = Vec::with_capacity(params.len());
            for param in params.iter() {
                match env.lookup_data_var(param) {
                    Some(ty) => { param_tys.push(ty); }
                    None => {
                        return Err(format!(
                            "ICE: Unable to look up type of function parameter: {}", param));
                    }
                }
            }
            Ok(FnTy(param_tys, box body_ty))
        }
        RecExpr(_) => { unimplemented!() }
        BlockExpr(ref stmts) => {
            // Infer for each value but the last one
            for stmt in stmts.init().iter() {
                try!(infer_stmt(env, stmt));
            }
            // Run the last one
            match stmts.last() {
                Some(&ExprStmt(ref expr)) => {
                    return infer_expr(env, expr);
                }
                Some(stmt) => {
                    try!(infer_stmt(env, stmt));
                }
                None => {}
            }
            // If the last element isn't an Expression, the value is Null ({})
            Ok(IdentTy(Ident(Atom::from_slice("Null"), BuiltIn)))
        }
    }
}

pub fn infer_stmt(env: &mut Environment, stmt: &Stmt) -> Result<(), String> {
    match *stmt {
        ExprStmt(ref expr) => {
            try!(infer_expr(env, expr));
            Ok(())
        }
        LetStmt(ref ident, ref expr) => {
            let ty = try!(infer_expr(env, expr));
            // TODO: Better error message on failure
            let ident = env.lookup_data_var(ident).unwrap();
            unify(env, &ident, &ty)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use il::*;
    use std::collections::HashMap;

    #[test]
    fn literals() {
        let stmts = vec![
            LetStmt(Ident::from_user_slice("id1"),
                    FnExpr(vec![Ident::from_user_slice("x1")],
                           box IdentExpr(Ident::from_user_slice("x1")))),
            LetStmt(Ident::from_user_slice("id2"),
                    FnExpr(vec![Ident::from_user_slice("x2")],
                           box IdentExpr(Ident::from_user_slice("x2")))),
            LetStmt(Ident::from_user_slice("x"),
                    CallExpr(FnCall(box IdentExpr(Ident::from_user_slice("id1")),
                                    vec![IdentExpr(Ident::from_user_slice("id2"))])))];
        info!("{}", stmts);
        // Create the environment - we don't have a easy way to do taht yet
        let mut env = Environment{
            data_vars: HashMap::new(),
            type_vars: HashMap::new(),
            counter: 0,
        };

        for ident in vec![Ident::from_user_slice("id1"),
                          Ident::from_user_slice("x1"),
                          Ident::from_user_slice("id2"),
                          Ident::from_user_slice("x2"),
                          Ident::from_user_slice("x")].iter() {
            let ty = env.introduce_type_var();
            env.data_vars.insert(ident.clone(), IdentTy(ty));
        }

        info!("{}", infer_expr(&mut env, &BlockExpr(stmts)));
        info!("{}", env);
    }
}
