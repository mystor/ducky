use string_cache::Atom;
use std::collections::{HashMap, HashSet};
use il::*;

use self::MaybeOwnedEnv::*;

#[deriving(Show, Clone)]
pub struct Environment {
    data_vars: HashMap<Ident, Ty>,
    type_vars: HashMap<Ident, Ty>,
    counter: uint,
}

impl Environment {
    // Accessors for the data from the environment
    fn lookup_type_var(&self, id: &Ident) -> Option<Ty> {
        self.type_vars.get(id).map(|x| { x.clone() })
    }

    fn lookup_data_var(&mut self, id: &Ident) -> Ty {
        if let Some(ty) = self.data_vars.get(id) {
            return ty.clone();
        }

        let ty = self.introduce_type_var();
        self.data_vars.insert(id.clone(), ty.clone());
        ty
    }
    
    // Creating a unique type variable
    fn introduce_type_var(&mut self) -> Ty {
        // TODO: Currently these names are awful
        let chars = "αβγδεζηθικλμνξοπρστυφχψωΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨω";
        let id = chars.slice_chars(self.counter % chars.len(), self.counter % chars.len() + 1);
        self.counter += 1;

        IdentTy(Ident(Atom::from_slice(id), Internal(self.counter)))
    }
    
    // Perform a substitution (bind the type variable id)
    fn substitute(&mut self, id: Ident, ty: Ty) {
        self.type_vars.insert(id, ty);
    }
    
}

// TODO: This can probably be merged into the Scope<'a> Struct
#[deriving(Show)]
enum MaybeOwnedEnv<'a> {
    OwnedEnv(Environment),
    SharedEnv(&'a mut Environment),
}

impl <'a> Deref<Environment> for MaybeOwnedEnv<'a> {
    fn deref<'a>(&'a self) -> &'a Environment {
        match *self {
            OwnedEnv(ref env) => env,
            SharedEnv(ref env) => &**env,
        }
    }
}

impl <'a> DerefMut<Environment> for MaybeOwnedEnv<'a> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Environment {
        match *self {
            OwnedEnv(ref mut env) => env,
            SharedEnv(ref mut env) => &mut **env,
        }
    }
}
    
#[deriving(Show)]
pub struct Scope<'a> {
    env: MaybeOwnedEnv<'a>,
    bound_type_vars: HashSet<Ident>,
}

impl <'a>Scope<'a> {
    pub fn new() -> Scope<'static> {
        // Type Variables
        let mut type_vars = HashMap::new();
        type_vars.insert(Ident::from_builtin_slice("Int"),
                         RecTy(box None, vec![MethodTyProp(Symbol::from_slice("+"),
                                                           vec![IdentTy(Ident::from_builtin_slice("Int"))],
                                                           IdentTy(Ident::from_builtin_slice("Int")))]));
                                                                      
        Scope{
            env: OwnedEnv(Environment{
                type_vars: type_vars,
                data_vars: HashMap::new(),
                counter: 0,
            }),
            bound_type_vars: HashSet::new(),
        }
    }
    fn new_child<'b>(&'b mut self, bound_type_vars: HashSet<Ident>) -> Scope<'b> {
        Scope{
            env: SharedEnv(self.env.deref_mut()),
            bound_type_vars: (self.bound_type_vars.clone().into_iter()
                              .chain(bound_type_vars.into_iter()).collect())
        }
    }
    
    fn is_bound_type_var(&self, id: &Ident) -> bool {
        self.bound_type_vars.contains(id) || self.lookup_type_var(id).is_some()
    }
    
    fn instantiate(&mut self, ty: &Ty, mappings: &mut HashMap<Ident, Ty>) -> Ty {
        match *ty {
            IdentTy(ref id) => {
                if self.is_bound_type_var(id) {
                    ty.clone()
                } else {
                    // This is an unbound variable, look up the name we have given
                    // it in the instance, or give it a new name
                    let lookup = mappings.get(id).map(|x| { x.clone() });
                    lookup.unwrap_or_else(|| {
                        let ty_var = self.introduce_type_var();
                        mappings.insert(id.clone(), ty_var.clone());
                        ty_var
                    })
                }
            }
            RecTy(ref extends, ref props) => {
                // Instantiate all of the properties!
                let extends = match *extends { // @TODO: Why can't I .map()?
                    box Some(ref extends) => Some(self.instantiate(extends, mappings)),
                    box None => None,
                };
                
                let props = props.iter().map(|prop| {
                    match *prop {
                        ValTyProp(ref symb, ref ty) => {
                            ValTyProp(symb.clone(), self.instantiate(ty, mappings))
                        }
                        MethodTyProp(ref symb, ref args, ref res) => {
                            let nargs = args.iter().map(|x| {
                                self.instantiate(x, mappings)
                            }).collect();
                            let nres = self.instantiate(res, mappings);
                            MethodTyProp(symb.clone(), nargs, nres)
                        }
                    }
                }).collect();

                RecTy(box extends, props)
            }
            FnTy(ref args, ref res) => {
                let nargs = args.iter().map(|x| { self.instantiate(x, mappings) }).collect();
                FnTy(nargs, box self.instantiate(&**res, mappings))
            }
        }
    }
}

impl <'a> DerefMut<Environment> for Scope<'a> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Environment {
        self.env.deref_mut()
    }
}

impl <'a> Deref<Environment> for Scope<'a> {
    fn deref<'a>(&'a self) -> &'a Environment {
        self.env.deref()
    }
}

fn unify_props(scope: &mut Scope, a: &TyProp, b: &TyProp) -> Result<(), String> {
    match (a, b) {
        (&ValTyProp(_, ref aty), &ValTyProp(_, ref bty)) => {
            unify(scope, aty, bty)
        }
        (&MethodTyProp(_, ref aargs, ref ares), &MethodTyProp(_, ref bargs, ref bres)) => {
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
    // Generate a set of substitutions such that a == b in scope
    match (a, b) {
        (&IdentTy(ref a), b) => {
            // Check if we can abort early due to a recursive decl
            if let IdentTy(ref b) = *b {
                if b == a { return Ok(()) }
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
        (a, &IdentTy(ref b)) => {
            // TODO: Check if I should do this
            if let Some(ref ty) = scope.lookup_type_var(b) {
                unify(scope, ty, a)
            } else {
                scope.substitute(b.clone(), a.clone());
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
                try!(unify(scope, aarg, barg));
            }
            
            // Unify the results
            unify(scope, &**ares, &**bres)
        }
        (&RecTy(ref aextends, ref aprops), &RecTy(ref bextends, ref bprops)) => {
            // Find the intersection between aprops and bprops
            let mut only_a = HashMap::new();
            let mut only_b = HashMap::new();
            let mut joint  = HashMap::new();

            for aprop in aprops.iter() {
                if let Some(bprop) = bprops.iter().find(|bprop| { aprop.symbol() == bprop.symbol() }) {
                    joint.insert(aprop.symbol().clone(), (aprop, bprop));
                } else {
                    only_a.insert(aprop.symbol().clone(), aprop);
                }
            }
            
            for bprop in bprops.iter() {
                if ! joint.contains_key(bprop.symbol()) {
                    only_b.insert(bprop.symbol().clone(), bprop);
                }
            }
            
            // Unify all of the common properties
            for &(aprop, bprop) in joint.values() {
                try!(unify_props(scope, aprop, bprop));
            }
            
            // Merge the remaining values into the other maps
            // @TODO: I have a sneaking suspicion that this is fundamentally incorrect...
            if ! only_a.is_empty() {
                if let box Some(ref ty) = *bextends {
                    try!(unify(scope, 
                               &RecTy(aextends.clone(),
                                      only_a.values().map(|x| x.clone().clone()).collect()),
                               ty));
                } else {
                    // @TODO: This error message is awful
                    return Err(format!("Cannot unify {} and {}", a, b));
                }
            }
            
            if ! only_b.is_empty() {
                if let box Some(ref ty) = *aextends {
                    try!(unify(scope, ty,
                               &RecTy(bextends.clone(),
                                      only_b.values().map(|x| x.clone().clone()).collect())));
                } else {
                    // @TODO: This error message is awful
                    return Err(format!("Cannot unify {} and {}", a, b));
                }
            }
            
            Ok(())
        }
        _ => {
            // TODO: This message itself should probably never be shown to
            // users of the compiler, it should be made more useful where
            // unify() is called.
            Err(format!("Cannot unify {} and {}", a, b))
        }
    }
}

fn infer_body(scope: &mut Scope, params: &Vec<Ident>, body: &Expr) -> Result<Ty, String> {
    let bound = { // Determine the list of variables which should be bound
        let transform = |x| {
            if let IdentTy(id) = scope.lookup_data_var(x) {
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
        LiteralExpr(ref lit) => { Ok(lit.ty()) } // We probably can just inline that
        IdentExpr(ref ident) => {
            let uninst = scope.lookup_data_var(ident);
            Ok(scope.instantiate(&uninst, &mut HashMap::new()))
        }
        CallExpr(FnCall(ref callee, ref params)) => {
            let callee_ty = try!(infer_expr(scope, &**callee));
            let mut param_tys = Vec::with_capacity(params.len());
            for param in params.iter() {
                match infer_expr(scope, param) {
                    Ok(ty) => { param_tys.push(ty); }
                    Err(err) => { return Err(err); }
                }
            }
            let beta = scope.introduce_type_var();
            // TODO: Vastly improve this error message
            try!(unify(scope, &callee_ty, &FnTy(param_tys, box beta.clone())));
            Ok(beta)
        }
        CallExpr(MethodCall(ref obj, ref symb, ref params)) => {
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
            let require_ty = RecTy(box Some(scope.introduce_type_var()),
                                   vec![MethodTyProp(symb.clone(), param_tys, res.clone())]);
            try!(unify(scope, &obj_ty, &require_ty));
            Ok(res)
        }
        FnExpr(ref params, ref body) => {
            let body_ty = try!(infer_body(scope, params, &**body));
            let mut param_tys = Vec::with_capacity(params.len());
            for param in params.iter() {
                param_tys.push(scope.lookup_data_var(param));
            }
            Ok(FnTy(param_tys, box body_ty))
        }
        RecExpr(ref props) => {
            let self_type = scope.introduce_type_var();

            let mut prop_tys = Vec::with_capacity(props.len());

            for prop in props.iter() {
                match *prop {
                    ValProp(ref symb, ref expr) => {
                        prop_tys.push(
                            ValTyProp(symb.clone(), try!(infer_expr(scope, expr))))
                    }
                    MethodProp(ref symb, ref params, ref body) => {
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
                            MethodTyProp(symb.clone(), param_tys, body_ty))
                    }
                }
            }
            
            Ok(RecTy(box None, prop_tys))
        }
        BlockExpr(ref stmts) => {
            // Infer for each value but the last one
            for stmt in stmts.init().iter() {
                try!(infer_stmt(scope, stmt));
            }
            // Run the last one
            match stmts.last() {
                Some(&ExprStmt(ref expr)) => {
                    return infer_expr(scope, expr);
                }
                Some(stmt) => {
                    try!(infer_stmt(scope, stmt));
                }
                None => {}
            }
            // If the last element isn't an Expression, the value is Null ({})
            Ok(IdentTy(Ident(Atom::from_slice("Null"), BuiltIn)))
        }
    }
}

pub fn infer_stmt(scope: &mut Scope, stmt: &Stmt) -> Result<(), String> {
    match *stmt {
        ExprStmt(ref expr) => {
            try!(infer_expr(scope, expr));
            Ok(())
        }
        LetStmt(ref ident, ref expr) => {
            let ty = try!(infer_expr(scope, expr));
            // TODO: Better error message on failure
            let ident = scope.lookup_data_var(ident);
            unify(scope, &ident, &ty)
        }
    }
}

pub fn infer_prgm(body: Vec<Stmt>) -> Result<Scope<'static>, String> {
    let mut scope = Scope::new();
    try!(infer_expr(&mut scope, &BlockExpr(body)));
    Ok(scope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use il::*;

    #[test]
    fn compose_id_with_itself() {
        let stmts = vec![
            LetStmt(Ident::from_user_slice("id"),
                    FnExpr(vec![Ident::from_user_slice("x")],
                           box IdentExpr(Ident::from_user_slice("x")))),
            LetStmt(Ident::from_user_slice("x"),
                    CallExpr(FnCall(box IdentExpr(Ident::from_user_slice("id")),
                                    vec![IdentExpr(Ident::from_user_slice("id"))])))];
        
        debug!("{}", infer_prgm(stmts));
    }
    
    #[test]
    fn basic_method_call() {
        let stmts = vec![
            LetStmt(Ident::from_user_slice("myfn"),
                    FnExpr(vec![Ident::from_user_slice("x")],
                           box BlockExpr(vec![
                               ExprStmt(CallExpr(MethodCall(box IdentExpr(Ident::from_user_slice("x")),
                                                            Symbol::from_slice("foo"),
                                                            vec![LiteralExpr(IntLit(5))])))])))
            ];
        
        debug!("{}", infer_prgm(stmts));
    }
    
    #[test]
    fn fail() {
        let stmts = vec![
            LetStmt(Ident::from_user_slice("something"),
                    FnExpr(vec![Ident::from_user_slice("x")],
                           box CallExpr(FnCall(box IdentExpr(Ident::from_user_slice("x")),
                                                        vec![LiteralExpr(IntLit(5))])))),
            LetStmt(Ident::from_user_slice("y"),
                    CallExpr(FnCall(box IdentExpr(Ident::from_user_slice("something")),
                                    vec![LiteralExpr(IntLit(6))])))
                ];

        debug!("{}", infer_prgm(stmts));
    }
}
