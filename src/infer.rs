use string_cache::Atom;
use std::collections::HashMap;
use il::*;

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
            if let Some(ref ty) = env.lookup_data_var(a).clone() {
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
            if let Some(ref ty) = env.lookup_data_var(b).clone() {
                unify(env, ty, a)
            } else {
                env.substitute(b.clone(), a.clone());
                Ok(())
            }
        }
        (&FnTy(ref aargs, ref ares), &FnTy(ref bargs, ref bres)) => {
            Ok(())
        }
        (&RecTy(_), &RecTy(_)) => { unimplemented!() }
        _ => {
            Err(format!("Cannot unify {} and {}", a, b))
        }
    }
}

