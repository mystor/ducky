use std::fmt;
use std::collections::HashMap;
use intern::Atom;
use il::*;
use self::env::{Scope, Env};

mod util;
mod env;
mod unify;

#[cfg(test)]
mod test;

#[derive(Clone)]
pub struct InferValue {
    pub data_vars: HashMap<Ident, Ty>,
    pub type_vars: HashMap<Ident, Ty>,
}

impl fmt::Debug for InferValue {
    fn fmt<'a>(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{{\n"));
        try!(write!(f, "  data_vars: {{\n"));
        for (id, ty) in self.data_vars.iter() {
            try!(write!(f, "    {:10}: {:?}\n", format!("{:?}", id), ty));
        }
        try!(write!(f, "  }}\n"));
        try!(write!(f, "  type_vars: {{\n"));
        for (id, ty) in self.type_vars.iter() {
            try!(write!(f, "    {:10}: {:?}\n", format!("{:?}", id), ty));
        }
        try!(write!(f, "  }}\n"));
        write!(f, "}}")
    }
}


fn infer_body(scope: &mut Scope, params: &Vec<Ident>, body: &Expr) -> Result<Ty, String> {
    let bound = params.iter().map(|x| {
        if let Ty::Ident(id) = scope.lookup_data_var(x) {
            id
        } else { unreachable!() }
    }).collect();

    scope.push_child(bound);
    let res = infer_expr(scope, body);
    scope.pop_child();

    res
}

pub fn infer_expr(scope: &mut Scope, e: &Expr) -> Result<Ty, String> {
    match *e {
        Expr::Literal(ref lit) => { Ok(util::val_ty(scope, lit.ty())) } // We probably can just inline that
        Expr::Ident(ref ident) => {
            let uninst = scope.lookup_data_var(ident);
            Ok(scope.instantiate(&uninst, &mut HashMap::new()))
        }
        Expr::Call(ref obj, ref symb, ref params) => {
            let obj_ty = try!(infer_expr(scope, &**obj));

            let mut param_tys = Vec::with_capacity(params.len());
            for param in params.iter() {
                match infer_expr(scope, param) {
                    Ok(ty) => { param_tys.push(ty); }
                    Err(err) => { return Err(err); }
                }
            }

            let res = scope.introduce_type_var();
            // The object must have the method with the correct type. UNIFY!
            let require_ty = Ty::Rec(Some(box scope.introduce_type_var()),
                                     vec![TyProp::Method(symb.clone(), param_tys, res.clone())]);
            try!(unify::unify(scope, &obj_ty, &require_ty));
            Ok(res)
        }
        Expr::Member(ref obj, ref symb) => {
            let obj_ty = try!(infer_expr(scope, &**obj));

            let ty = scope.introduce_type_var();

            let require_ty = Ty::Rec(Some(box scope.introduce_type_var()),
                                     vec![TyProp::Val(symb.clone(), ty.clone())]);
            try!(unify::unify(scope, &obj_ty, &require_ty));

            Ok(ty)
        }
        Expr::Rec(ref props) => {
            let self_type = scope.introduce_type_var();

            let mut prop_tys = Vec::with_capacity(props.len());

            for prop in props.iter() {
                match *prop {
                    Prop::Val(ref symb, ref expr) => {
                        prop_tys.push(
                            TyProp::Val(symb.clone(), try!(infer_expr(scope, expr))))
                    }
                    Prop::Method(ref symb, ref params, ref body) => {
                        // Unify the first variable's type with self_type
                        // TODO: Do this at the end?
                        let first_type = scope.lookup_data_var(&params[0]);
                        try!(unify::unify(scope, &first_type, &self_type));

                        let body_ty = try!(infer_body(scope, params, body));
                        let mut param_tys = Vec::with_capacity(params.len());
                        for param in params.iter() {
                            param_tys.push(scope.lookup_data_var(param));
                        }
                        prop_tys.push(
                            TyProp::Method(symb.clone(), param_tys, body_ty))
                    }
                }
            }

            Ok(util::val_ty(scope, Ty::Rec(None, prop_tys)))
        }
        Expr::Block(ref stmts) => {
            // Infer for each value but the last one
            for stmt in stmts.init().iter() {
                try!(infer_stmt(scope, stmt));
            }
            // Run the last one
            match stmts.last() {
                Some(&Stmt::Expr(ref expr)) => {
                    return infer_expr(scope, expr);
                }
                Some(stmt) => {
                    try!(infer_stmt(scope, stmt));
                }
                None => {}
            }
            // If the last element isn't an Expression, the value is Null ({})
            Ok(util::val_ty(scope, Ty::Ident(Ident(Atom::from_slice("Null"), BuiltIn))))
        }
        Expr::If(box ref cond, box ref thn, box ref els) => {
            // Infer the type of the condition, and ensure it is Bool
            let cond_ty = try!(infer_expr(scope, cond));
            try!(unify::unify(scope, &cond_ty, &Ty::Ident(Ident(Atom::from_slice("Bool"), BuiltIn))));

            // Infer the type of the different branches
            let thn_ty = try!(infer_expr(scope, thn));
            let els_ty = if let Some(ref els_expr) = *els {
                try!(infer_expr(scope, els_expr))
            } else {
                Ty::Ident(Ident(Atom::from_slice("Null"), BuiltIn))
            };

            // Both branches currently need to return the same type. We hope to
            // change that at some point by introducing sum types! Woo!
            // try!(unify::unify(&mut **scope, &thn_ty, &els_ty));

            Ok(Ty::Union(vec![thn_ty, els_ty]))
        }
    }
}

pub fn infer_stmt(scope: &mut Scope, stmt: &Stmt) -> Result<(), String> {
    match *stmt {
        Stmt::Expr(ref expr) => {
            try!(infer_expr(scope, expr));
            Ok(())
        }
        Stmt::Let(ref ident, ref expr) => {
            let ty = try!(infer_expr(scope, expr));
            // TODO: Better error message on failure
            let ident = scope.lookup_data_var(ident);
            unify::unify(scope, &ident, &ty)
        }
        Stmt::Empty => Ok(())
    }
}

pub fn infer_program(body: Vec<Stmt>) -> Result<InferValue, String> {
    let mut scope = Scope::new();
    try!(infer_expr(&mut scope, &Expr::Block(body)));
    Ok(scope.as_infervalue())
}
