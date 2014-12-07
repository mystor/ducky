use string_cache::Atom;
use std::fmt;
use std::collections::{HashMap, HashSet};
use il::*;

use self::MaybeOwnedEnv::*;

#[cfg(test)]
pub mod test;

#[deriving(Clone)]
pub struct InferValue {
    pub data_vars: HashMap<Ident, Ty>,
    pub type_vars: HashMap<Ident, Ty>,
}

impl fmt::Show for InferValue {
    fn fmt<'a>(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{{\n"));
        try!(write!(f, "  data_vars: {{\n"));
        for (id, ty) in self.data_vars.iter() {
            try!(write!(f, "    {:10}: {}\n", format!("{}", id), ty));
        }
        try!(write!(f, "  }}\n"));
        try!(write!(f, "  type_vars: {{\n"));
        for (id, ty) in self.type_vars.iter() {
            try!(write!(f, "    {:10}: {}\n", format!("{}", id), ty));
        }
        try!(write!(f, "  }}\n"));
        write!(f, "}}")
    }
}

#[deriving(Show, Clone)]
pub struct Environment {
    data_vars: HashMap<Ident, Ty>,
    type_vars: HashMap<Ident, Ty>,
    unified: HashSet<(Ty, Ty)>,
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
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let id = chars.slice_chars(self.counter % chars.len(), self.counter % chars.len() + 1);
        self.counter += 1;

        Ty::Ident(Ident(Atom::from_slice(id), Internal(self.counter)))
    }

    // Perform a substitution (bind the type variable id)
    fn substitute(&mut self, id: Ident, ty: Ty) {
        self.type_vars.insert(id, ty);
    }

    // Produce an InferValue
    fn as_infervalue(&self) -> InferValue {
        InferValue {
            data_vars: self.data_vars.clone(),
            type_vars: self.type_vars.clone(),
        }
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
                         Ty::Rec(box None, vec![TyProp::Method(Symbol::from_slice("+"),
                                                               vec![Ty::Ident(Ident::from_builtin_slice("Int"))],
                                                               Ty::Ident(Ident::from_builtin_slice("Int"))),
                                                TyProp::Method(Symbol::from_slice("*"),
                                                               vec![Ty::Ident(Ident::from_builtin_slice("Int"))],
                                                               Ty::Ident(Ident::from_builtin_slice("Int")))]));

        Scope{
            env: OwnedEnv(Environment{
                type_vars: type_vars,
                data_vars: HashMap::new(),
                unified: HashSet::new(),
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

    fn instantiate(&mut self, ty: &Ty, mappings: &mut HashMap<Ident, Ty>) -> Ty {
        match *ty {
            Ty::Ident(ref id) => {
                if let Some(ty) = mappings.get(id) {
                    // Check if this identifier has already been looked up in the mappings
                    return ty.clone()
                }

                if self.bound_type_vars.contains(id) {
                    // Bound type vars are explicitly not initialized
                    ty.clone()
                } else {
                    // Create a type var to represent the instantiated version
                    let ty_var = self.introduce_type_var();
                    mappings.insert(id.clone(), ty_var.clone());

                    if let Some(ref ty) = self.lookup_type_var(id) {
                        // Instantiate the type which is being pointed to
                        let instantiated = self.instantiate(ty, mappings);

                        // Make the ty_var point to the instantiated type
                        self.substitute(
                            ty_var.unwrap_ident(),
                            instantiated);
                    }

                    ty_var
                }
            }
            Ty::Rec(ref extends, ref props) => {
                // Instantiate all of the properties!
                let extends = match *extends { // @TODO: Why can't I .map()?
                    box Some(ref extends) => Some(self.instantiate(extends, mappings)),
                    box None => None,
                };

                let props = props.iter().map(|prop| {
                    match *prop {
                        TyProp::Val(ref symb, ref ty) => {
                            TyProp::Val(symb.clone(), self.instantiate(ty, mappings))
                        }
                        TyProp::Method(ref symb, ref args, ref res) => {
                            let nargs = args.iter().map(|x| {
                                self.instantiate(x, mappings)
                            }).collect();
                            let nres = self.instantiate(res, mappings);
                            TyProp::Method(symb.clone(), nargs, nres)
                        }
                    }
                }).collect();

                Ty::Rec(box extends, props)
            }
            Ty::Fn(ref args, ref res) => {
                let nargs = args.iter().map(|x| { self.instantiate(x, mappings) }).collect();
                Ty::Fn(nargs, box self.instantiate(&**res, mappings))
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

    // Record the previously unified values in the scope, and abort with Ok(()) if they have been unified before
    let ty_pairs = (a.clone(), b.clone());
    if scope.unified.contains(&(a.clone(), b.clone())) {
        return Ok(());
    } else {
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
        (&Ty::Fn(ref aargs, ref ares), &Ty::Fn(ref bargs, ref bres)) => {
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
        (&Ty::Rec(box ref _aextends, ref _aprops), &Ty::Rec(box ref _bextends, ref _bprops)) => {
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

            fn expand_extends<'a>(scope: &mut Scope<'a>, extends: &mut Option<Ty>, props: &mut HashMap<Symbol, TyProp>) {
                loop {
                    let mut new_extends;

                    match *extends {
                        Some(Ty::Ident(ref ident)) => {
                            // We are extending an identifier, let's expand it!
                            if let Some(ty) = scope.lookup_type_var(ident) {
                                new_extends = Some(ty);
                            } else {
                                // We are looking at a wildcard! woo!
                                break;
                            }
                        }
                        Some(Ty::Rec(box ref nextends, ref nprops)) => {
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
            if let Some(Ty::Ident(ref ident)) = bextends {
                // We need to unify bextends with something
                scope.substitute(ident.clone(),
                                 Ty::Rec(box Some(common_free.clone()),
                                         only_a.values().map(|x| (**x).clone()).collect()));
            } else if ! only_a.is_empty() {
                return Err(format!("Cannot unify {} and {}", a, b));
            }

            // Merge the remaining values into the other maps
            if let Some(Ty::Ident(ref ident)) = aextends {
                // We need to unify bextends with something
                scope.substitute(ident.clone(),
                                 Ty::Rec(box Some(common_free.clone()),
                                         only_b.values().map(|x| (**x).clone()).collect()));
            } else if ! only_b.is_empty() {
                return Err(format!("Cannot unify {} and {}", a, b));
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
        Expr::Call(Call::Fn(ref callee, ref params)) => {
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
            try!(unify(scope, &callee_ty, &Ty::Fn(param_tys, box beta.clone())));
            Ok(beta)
        }
        Expr::Call(Call::Method(ref obj, ref symb, ref params)) => {
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
            let require_ty = Ty::Rec(box Some(scope.introduce_type_var()),
                                     vec![TyProp::Method(symb.clone(), param_tys, res.clone())]);
            try!(unify(scope, &obj_ty, &require_ty));
            Ok(res)
        }
        Expr::Member(ref obj, ref symb) => {
            let obj_ty = try!(infer_expr(scope, &**obj));

            let ty = scope.introduce_type_var();

            let require_ty = Ty::Rec(box Some(scope.introduce_type_var()),
                                     vec![TyProp::Val(symb.clone(), ty.clone())]);
            try!(unify(scope, &obj_ty, &require_ty));

            Ok(ty)
        }
        Expr::Fn(ref params, ref body) => {
            let body_ty = try!(infer_body(scope, params, &**body));
            let mut param_tys = Vec::with_capacity(params.len());
            for param in params.iter() {
                param_tys.push(scope.lookup_data_var(param));
            }
            Ok(Ty::Fn(param_tys, box body_ty))
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

            Ok(Ty::Rec(box None, prop_tys))
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
        Expr::Match(_, ref options) => {
            // unify target with union of option types
            // unless option types includes wildcard
            for _ in options.iter() {
            }
            Ok(Ty::Ident(Ident(Atom::from_slice("Null"), BuiltIn)))
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
