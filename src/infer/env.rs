use std::collections::{HashMap, HashSet};
use string_cache::Atom;
use il::*;
use infer::InferValue;

/// A struct implementing Env has access to a set of type_vars.
/// At some point, Env will probably be extended to include most
/// of the functionality of Environment and Scope.
pub trait Env {
    fn lookup_type_var(&self, id: &Ident) -> Option<&Ty>;

    fn lookup_data_var(&mut self, id: &Ident) -> Ty;

    fn introduce_type_var(&mut self) -> Ty;

    fn substitute(&mut self, id: Ident, ty: Ty);
}

#[deriving(Show, Clone)]
struct Environment {
    data_vars: HashMap<Ident, Ty>,
    type_vars: HashMap<Ident, Ty>,
    counter: uint,
}

// TODO: This can probably be merged into the Scope<'a> Struct
#[deriving(Show)]
enum MOE<'a> {
    Owned(Environment),
    Shared(&'a mut Environment),
}

impl <'a> Deref<Environment> for MOE<'a> {
    fn deref<'a>(&'a self) -> &'a Environment {
        match *self {
            MOE::Owned(ref env) => env,
            MOE::Shared(ref env) => &**env,
        }
    }
}

impl <'a> DerefMut<Environment> for MOE<'a> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Environment {
        match *self {
            MOE::Owned(ref mut env) => env,
            MOE::Shared(ref mut env) => &mut **env,
        }
    }
}

#[deriving(Show)]
pub struct Scope<'a> {
    env: MOE<'a>,
    bound_type_vars: HashSet<Ident>,
}

impl <'a>Scope<'a> {
    pub fn new() -> Scope<'static> {
        // Type Variables
        let mut type_vars = HashMap::new();

        // TODO: Builtins shouldn't be declared here, this is very sloppy.
        // we should have a better way of declaring the builtins
        // (possibly using macros so that we can use nice syntax?)
        type_vars.insert(Ident::from_builtin_slice("Int"),
                         Ty::Rec(None, vec![TyProp::Method(Symbol::from_slice("+"),
                                                           vec![Ty::Ident(Ident::from_builtin_slice("Int"))],
                                                           Ty::Ident(Ident::from_builtin_slice("Int"))),
                                            TyProp::Method(Symbol::from_slice("*"),
                                                           vec![Ty::Ident(Ident::from_builtin_slice("Int"))],
                                                           Ty::Ident(Ident::from_builtin_slice("Int")))]));

        Scope{
            env: MOE::Owned(Environment{
                type_vars: type_vars,
                data_vars: HashMap::new(),
                counter: 0,
            }),
            bound_type_vars: HashSet::new(),
        }
    }
    pub fn new_child<'b>(&'b mut self, bound_type_vars: HashSet<Ident>) -> Scope<'b> {
        Scope{
            env: MOE::Shared(self.env.deref_mut()),
            bound_type_vars: (self.bound_type_vars.clone().into_iter()
                              .chain(bound_type_vars.into_iter()).collect())
        }
    }

    pub fn instantiate(&mut self, ty: &Ty, mappings: &mut HashMap<Ident, Ty>) -> Ty {
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

                    if let Some(ref ty) = self.lookup_type_var(id).map(|x| x.clone()) {
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
                let extends = extends.as_ref().map(|x| box self.instantiate(&**x, mappings));

                let props = props.iter().map(|prop| {
                    match *prop {
                        TyProp::Val(ref symb, ref ty) => {
                            TyProp::Val(symb.clone(), self.instantiate(ty, mappings))
                        }
                        TyProp::Method(ref symb, ref args, ref res) => {
                            let nargs = args.iter().map(|x| self.instantiate(x, mappings)).collect();
                            let nres = self.instantiate(res, mappings);
                            TyProp::Method(symb.clone(), nargs, nres)
                        }
                    }
                }).collect();

                Ty::Rec(extends, props)
            }
            Ty::Union(ref options) => {
                let nopts = options.iter().map(|x| self.instantiate(x, mappings)).collect();
                Ty::Union(nopts)
            }
        }
    }

    pub fn as_infervalue(&self) -> InferValue {
        // TODO: Remove
        InferValue{
            data_vars: self.env.data_vars.clone(),
            type_vars: self.env.type_vars.clone(),
        }
    }
}

impl <'a> Env for Scope<'a> {
    fn lookup_type_var(&self, id: &Ident) -> Option<&Ty> {
        self.env.type_vars.get(id)
    }

    fn lookup_data_var(&mut self, id: &Ident) -> Ty {
        if let Some(ty) = self.env.data_vars.get(id) {
            return ty.clone();
        }

        let ty = self.introduce_type_var();
        self.env.data_vars.insert(id.clone(), ty.clone());
        ty
    }

    // Creating a unique type variable
    fn introduce_type_var(&mut self) -> Ty {
        // TODO: Currently these names are awful
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let id = chars.slice_chars(self.env.counter % chars.len(), self.env.counter % chars.len() + 1);
        self.env.counter += 1;

        Ty::Ident(Ident(Atom::from_slice(id), Internal(self.env.counter)))
    }

    // Perform a substitution (bind the type variable id)
    // id _must_ be unbound at the point of substitution
    fn substitute(&mut self, id: Ident, ty: Ty) {
        let prev = self.env.type_vars.insert(id, ty);
        assert!(prev.is_none());
    }
}
