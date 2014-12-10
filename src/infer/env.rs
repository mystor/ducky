use std::fmt;
use std::collections::{HashMap, HashSet};
use string_cache::Atom;
use il::*;

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
    pub data_vars: HashMap<Ident, Ty>,
    pub type_vars: HashMap<Ident, Ty>,
    pub unified: HashSet<(Ty, Ty)>,
    counter: uint,
}

impl Environment {
    // Accessors for the data from the environment
    pub fn lookup_type_var(&self, id: &Ident) -> Option<&Ty> {
        self.type_vars.get(id)
    }

    pub fn lookup_data_var(&mut self, id: &Ident) -> Ty {
        if let Some(ty) = self.data_vars.get(id) {
            return ty.clone();
        }

        let ty = self.introduce_type_var();
        self.data_vars.insert(id.clone(), ty.clone());
        ty
    }

    // Creating a unique type variable
    pub fn introduce_type_var(&mut self) -> Ty {
        // TODO: Currently these names are awful
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let id = chars.slice_chars(self.counter % chars.len(), self.counter % chars.len() + 1);
        self.counter += 1;

        Ty::Ident(Ident(Atom::from_slice(id), Internal(self.counter)))
    }

    // Perform a substitution (bind the type variable id)
    pub fn substitute(&mut self, id: Ident, ty: Ty) {
        self.type_vars.insert(id, ty);
    }

    // Produce an InferValue
    pub fn as_infervalue(&self) -> InferValue {
        InferValue {
            data_vars: self.data_vars.clone(),
            type_vars: self.type_vars.clone(),
        }
    }
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
                unified: HashSet::new(),
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
                let extends = match *extends { // @TODO: Why can't I .map()?
                    Some(box ref extends) => Some(box self.instantiate(extends, mappings)),
                    None => None,
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

                Ty::Rec(extends, props)
            }
            Ty::Union(ref options) => {
                let nopts = options.iter().map(|x| self.instantiate(x, mappings)).collect();
                Ty::Union(nopts)
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
