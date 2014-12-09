use string_cache::Atom;
use std::collections::HashMap;
use il::*;
use self::env::Scope;
pub use self::env::InferValue;

mod env;
#[cfg(test)]
mod test;

fn unify_props(scope: &mut Scope, a: &TyProp, b: &TyProp) -> Result<(), String> {
    match (a, b) {
        (&TyProp::Val(_, ref aty), &TyProp::Val(_, ref bty)) => {
            unify(scope, aty, bty)
        }
        (&TyProp::Method(_, ref aargs, ref ares), &TyProp::Method(_, ref bargs, ref bres)) => {
            if aargs.len() != bargs.len() {
                return Err(format!("Cannot unify {} and {}", a, b));
            }

            // Unify each of the arguments
            for (aarg, barg) in aargs.iter().zip(bargs.iter()) {
                try!(unify(scope, aarg, barg));
            }

            unify(scope, ares, bres)
        }
        _ => {
            Err(format!("Cannot unify properties: {} and {}", a, b))
        }
    }
}

pub fn unify(scope: &mut Scope, a: &Ty, b: &Ty) -> Result<(), String> {
    debug!("Unifying {} <=> {}", a, b);
    debug!("Environment: {}", "{");
    for (key, value) in scope.type_vars.iter() {
        debug!("  {}: {}", key, value);
    }
    debug!("{}", "}");

    // Record the previously unified values in the scope,
    // and abort with Ok(()) if they have been unified before
    let ty_pairs = (a.clone(), b.clone());
    if scope.unified.contains(&(a.clone(), b.clone())) {
        return Ok(());
    } else {
        // If they haven't been unified before, assume that they have!
        scope.unified.insert(ty_pairs);
    }

    // Generate a set of substitutions such that a == b in scope
    match (a, b) {
        (&Ty::Ident(ref a), b) => {
            // Check if we can abort early due to a recursive decl
            if let Ty::Ident(ref b) = *b {
                if b == a { return Ok(()) }

                // If both are identifiers, and the second is unbound, substitute!
                if let None = scope.lookup_type_var(b) {
                    scope.substitute(b.clone(), Ty::Ident(a.clone()));
                    return Ok(());
                }
            }

            if let Some(ref ty) = scope.lookup_type_var(a) {
                // The type name is explicit, resolve it
                unify(scope, ty, b)
            } else {
                // The type name is unbounded, substitute it for b
                scope.substitute(a.clone(), b.clone());
                Ok(())
            }
        }
        (a, &Ty::Ident(ref b)) => {
            // TODO: Check if I should do this
            if let Some(ref ty) = scope.lookup_type_var(b) {
                unify(scope, ty, a)
            } else {
                scope.substitute(b.clone(), a.clone());
                Ok(())
            }
        }
        (&Ty::Rec(ref _aextends, ref _aprops), &Ty::Rec(ref _bextends, ref _bprops)) => {
            let mut aextends = _aextends.clone();
            let mut aprops = HashMap::new();
            let mut bextends = _bextends.clone();
            let mut bprops = HashMap::new();

            for aprop in _aprops.iter() {
                aprops.insert(aprop.symbol().clone(), aprop.clone());
            }

            for bprop in _bprops.iter() {
                bprops.insert(bprop.symbol().clone(), bprop.clone());
            }

            fn expand_extends<'a>(scope: &mut Scope<'a>, extends: &mut Option<Box<Ty>>, props: &mut HashMap<Symbol, TyProp>) {
                loop {
                    let mut new_extends;

                    match *extends {
                        Some(box Ty::Ident(ref ident)) => {
                            // We are extending an identifier, let's expand it!
                            if let Some(ty) = scope.lookup_type_var(ident) {
                                new_extends = Some(box ty);
                            } else {
                                // We are looking at a wildcard! woo!
                                break;
                            }
                        }
                        Some(box Ty::Rec(ref nextends, ref nprops)) => {
                            // We are extending a record, merge it in!
                            for prop in nprops.iter() {
                                assert!(props.insert(prop.symbol().clone(), prop.clone()).is_none());
                            }
                            new_extends = nextends.clone()
                        }
                        None => {
                            // We have reached a concrete type!
                            break;
                        }
                        Some(_) => panic!("You can't extend a function?!? what?"), // @TODO: Improve
                    }

                    *extends = new_extends;
                }
            }

            expand_extends(scope, &mut aextends, &mut aprops);
            expand_extends(scope, &mut bextends, &mut bprops);

            // Find the intersection between aprops and bprops
            let mut only_a = HashMap::new();
            let mut only_b = HashMap::new();
            let mut joint  = HashMap::new();

            for aprop in aprops.values() {
                if let Some(bprop) = bprops.values().find(|bprop| { aprop.symbol() == bprop.symbol() }) {
                    joint.insert(aprop.symbol().clone(), (aprop, bprop));
                } else {
                    only_a.insert(aprop.symbol().clone(), aprop);
                }
            }

            for bprop in bprops.values() {
                if ! joint.contains_key(bprop.symbol()) {
                    only_b.insert(bprop.symbol().clone(), bprop);
                }
            }

            // Unify all of the common properties
            for &(aprop, bprop) in joint.values() {
                try!(unify_props(scope, aprop, bprop));
            }

            let common_free = scope.introduce_type_var();

            // Merge the remaining values into the other maps
            if let Some(box Ty::Ident(ref ident)) = bextends {
                // We need to unify bextends with something
                scope.substitute(ident.clone(),
                                 Ty::Rec(Some(box common_free.clone()),
                                         only_a.values().map(|x| (**x).clone()).collect()));
            } else if ! only_a.is_empty() {
                return Err(format!("Cannot unify {} and {}", a, b));
            }

            // Merge the remaining values into the other maps
            if let Some(box Ty::Ident(ref ident)) = aextends {
                // We need to unify bextends with something
                scope.substitute(ident.clone(),
                                 Ty::Rec(Some(box common_free.clone()),
                                         only_b.values().map(|x| (**x).clone()).collect()));
            } else if ! only_b.is_empty() {
                return Err(format!("Cannot unify {} and {}", a, b));
            }

            Ok(())
        }
        (&Ty::Union(ref aopts), &Ty::Union(ref bopts)) => {
            unimplemented!()
        }
        (&Ty::Union(_), &Ty::Rec(_, _)) => {
            unimplemented!()
        }
        (&Ty::Rec(_, _), &Ty::Union(_)) => {
            unimplemented!()
        }
    }
}

fn infer_body(scope: &mut Scope, params: &Vec<Ident>, body: &Expr) -> Result<Ty, String> {
    let bound = { // Determine the list of variables which should be bound
        let transform = |x| {
            if let Ty::Ident(id) = scope.lookup_data_var(x) {
                id
            } else { unreachable!() }
        };
        params.iter().map(transform).collect()
    };

    let mut new_scope = scope.new_child(bound);
    infer_expr(&mut new_scope, body)
}

pub fn infer_expr(scope: &mut Scope, e: &Expr) -> Result<Ty, String> {
    match *e {
        Expr::Literal(ref lit) => { Ok(lit.ty()) } // We probably can just inline that
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
            try!(unify(scope, &obj_ty, &require_ty));
            Ok(res)
        }
        Expr::Member(ref obj, ref symb) => {
            let obj_ty = try!(infer_expr(scope, &**obj));

            let ty = scope.introduce_type_var();

            let require_ty = Ty::Rec(Some(box scope.introduce_type_var()),
                                     vec![TyProp::Val(symb.clone(), ty.clone())]);
            try!(unify(scope, &obj_ty, &require_ty));

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
                        try!(unify(scope, &first_type, &self_type));

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

            Ok(Ty::Rec(None, prop_tys))
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
            Ok(Ty::Ident(Ident(Atom::from_slice("Null"), BuiltIn)))
        }
        Expr::If(box ref cond, box ref thn, box ref els) => {
            // Infer the type of the condition, and ensure it is Bool
            let cond_ty = try!(infer_expr(scope, cond));
            try!(unify(scope, &cond_ty, &Ty::Ident(Ident(Atom::from_slice("Bool"), BuiltIn))));

            // Infer the type of the different branches
            let thn_ty = try!(infer_expr(scope, thn));
            let els_ty = if let Some(ref els_expr) = *els {
                try!(infer_expr(scope, els_expr))
            } else {
                Ty::Ident(Ident(Atom::from_slice("Null"), BuiltIn))
            };

            // Both branches currently need to return the same type. We hope to
            // change that at some point by introducing sum types! Woo!
            try!(unify(scope, &thn_ty, &els_ty));

            Ok(thn_ty)
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
            unify(scope, &ident, &ty)
        }
        Stmt::Empty => Ok(())
    }
}

pub fn infer_program(body: Vec<Stmt>) -> Result<InferValue, String> {
    let mut scope = Scope::new();
    try!(infer_expr(&mut scope, &Expr::Block(body)));
    Ok(scope.as_infervalue())
}
