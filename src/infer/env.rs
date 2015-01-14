use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::collections::{HashMap, HashSet};
use intern::Atom;
use il::*;
use infer::util::free_vars;
use infer::InferValue;

/// A struct implementing Env has access to a set of type_vars.
/// At some point, Env will probably be extended to include most
/// of the functionality of Environment and Scope.
pub trait Env {
    fn lookup_type_var(&self, id: &Ident) -> Option<&Ty>;

    fn lookup_data_var(&mut self, id: &Ident) -> Ty;

    fn introduce_type_var(&mut self) -> Ty;

    fn substitute(&mut self, id: Ident, ty: Ty);

    fn as_infervalue(&self) -> InferValue;
}

#[derive(Show)]
pub struct Scope {
    data_vars: HashMap<Ident, Ty>,
    type_vars: HashMap<Ident, Ty>,
    counter: u32,

    bound_vars: Vec<HashSet<Ident>>,
}

impl Scope {
    pub fn new() -> Scope {
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

        Scope {
            type_vars: type_vars,
            data_vars: HashMap::new(),
            counter: 0,

            bound_vars: vec![HashSet::new()],
        }
    }

    pub fn push_child(&mut self, bound_vars: HashSet<Ident>) {
        // Also add all free variables in the bound vars to the list of bound vars!
        let bvs = bound_vars.iter().fold(HashSet::new(), |mut v, bv| {
            v.extend(free_vars(self, &Ty::Ident(bv.clone())).iter().cloned());
            v
        });

        self.bound_vars.push(bvs);
    }

    pub fn pop_child(&mut self) {
        self.bound_vars.pop();
    }

    fn is_bound(&self, var: &Ident) -> bool {
        // Theoretically should be done in reverse, but makes no difference
        self.bound_vars.iter().any(|bv| bv.contains(var))
    }

    pub fn instantiate(&mut self, ty: &Ty, mappings: &mut HashMap<Ident, Ty>) -> Ty {
        match *ty {
            Ty::Ident(ref id) => {
                if let Some(ty) = mappings.get(id) {
                    // It's been handled already, just go with it!
                    return ty.clone()
                } /* else */
                if self.is_bound(id) {
                    // Bound type vars are explicitly not initialized
                    ty.clone()
                } else {
                    // Create a type var to represent the instantiated version
                    let ty_var = self.introduce_type_var();
                    mappings.insert(id.clone(), ty_var.clone());

                    if let Some(ty) = self.lookup_type_var(id).cloned() {
                        // Instantiate the type which is being pointed to
                        let instantiated = self.instantiate(&ty, mappings);

                        // Make the ty_var point to the instantiated type
                        self.substitute(ty_var.unwrap_ident(), instantiated);
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

    fn maybe_bind(&mut self, id: &Ident, maybe_binds: &[Ident]) {
        for bv in self.bound_vars.iter_mut().rev() {
            if bv.contains(id) {
                bv.extend(maybe_binds.iter().cloned());
                return;
            }
        }

        panic!()
    }
}

impl Env for Scope {
    fn lookup_type_var(&self, id: &Ident) -> Option<&Ty> {
        self.type_vars.get(id)
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
        let id = chars.slice_chars(self.counter as usize % chars.len(),
                                   self.counter as usize % chars.len() + 1);
        self.counter += 1;

        Ty::Ident(Ident(Atom::from_slice(id), Internal(self.counter)))
    }

    // Perform a substitution (bind the type variable id)
    // id _must_ be unbound at the point of substitution
    fn substitute(&mut self, id: Ident, ty: Ty) {
        // Substitute the type variable
        let prev = self.type_vars.insert(id.clone(), ty.clone());

        if self.is_bound(&id) {
            // Determine what new variables have to be bound
            let free = free_vars(self, &ty);
            let newly_bound: Vec<_> = free.iter().filter(|x| {
                ! self.is_bound(*x)
            }).cloned().collect();

            self.maybe_bind(&id, newly_bound.as_slice());
        }

        assert!(prev.is_none());
    }

    fn as_infervalue(&self) -> InferValue {
        // TODO: Remove
        InferValue{
            data_vars: self.data_vars.clone(),
            type_vars: self.type_vars.clone(),
        }
    }
}
