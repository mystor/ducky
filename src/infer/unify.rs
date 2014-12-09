use std::collections::HashMap;
use infer::Scope;
use il::*;

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

/// Reduces the type to it's "standard form". In this form, all
/// "root" Ty::Idents are not bound in the scope, and there are no
/// nested unions.
fn std_form(scope: &mut Scope, ty: Ty) -> Ty {
    match ty {
        Ty::Ident(ref id) => {
            if let Some(nty) = scope.lookup_type_var(id) {
                std_form(scope, nty.clone())
            } else { ty.clone() }
        }
        Ty::Rec(ref extends, ref props) => {
            if let Some(box ref extends) = *extends {
                let nextends = std_form(scope, extends.clone());

                match nextends {
                    Ty::Ident(_) => {
                        Ty::Rec(Some(box nextends), props.clone())
                    }
                    Ty::Rec(ref extends, ref nprops) => {
                        // XXX: Invariant. props & nprops must not contain common symbols!
                        Ty::Rec(extends.clone(),
                                props.iter().chain(
                                    nprops.iter()).map(|x| x.clone()).collect())
                    }
                    Ty::Union(ref opts) => {
                        // Add props to every option!
                        let opts = opts.iter().map(|opt| {
                            std_form(scope,
                                     Ty::Rec(Some(box opt.clone()), props.clone()))
                        }).collect();

                        // TODO: Expand inner unions! (may be unnecessary here)

                        Ty::Union(opts)
                    }
                }
            } else { ty.clone() }
        }
        Ty::Union(ref opts) => {
            let opts = opts.iter().map(|opt| {
                std_form(scope, opt.clone())
            }).collect();
            // TODO: Expand inner unions!

            Ty::Union(opts)
        }
    }
}

pub fn unify(scope: &mut Scope, a: &Ty, b: &Ty) -> Result<(), String> {
    // Types in this language are very simple, they all take the form of records, or
    // unions of records. Which is going to be nice for us.
    // We need to first reduce both type a and type b to standard form, and then
    // unify them in standard form.
    let a = std_form(scope, a.clone());
    let b = std_form(scope, b.clone());

    match (&a, &b) {
        (&Ty::Ident(ref a), b) => {
            scope.substitute(a.clone(), b.clone());

            Ok(())
        }
        (a, &Ty::Ident(ref b)) => {
            scope.substitute(b.clone(), a.clone());

            Ok(())
        }
        (&Ty::Rec(ref aextends, ref aprops), &Ty::Rec(ref bextends, ref bprops)) => {
            let mut only_a = HashMap::new();
            let mut only_b = HashMap::new();
            let mut joint = HashMap::new();

            for aprop in aprops.iter() {
                only_a.insert(aprop.symbol().clone(), aprop);
            }
            for bprop in bprops.iter() {
                if let Some(aprop) = only_a.remove(bprop.symbol()) {
                    joint.insert(bprop.symbol().clone(), (aprop, bprop));
                } else {
                    only_b.insert(bprop.symbol().clone(), bprop);
                }
            }

            for &(aprop, bprop) in joint.values() {
                try!(unify_props(scope, aprop, bprop));
            }

            let common_free = scope.introduce_type_var();

            if let Some(box Ty::Ident(ref ident)) = *bextends {
                // We need to unify bextends with something
                scope.substitute(ident.clone(),
                                 Ty::Rec(Some(box common_free.clone()),
                                         only_a.values().map(|x| (**x).clone()).collect()));
            } else if ! only_a.is_empty() {
                return Err(format!("Cannot unify {} and {}", a, b));
            }

            // Merge the remaining values into the other maps
            if let Some(box Ty::Ident(ref ident)) = *aextends {
                // We need to unify bextends with something
                scope.substitute(ident.clone(),
                                 Ty::Rec(Some(box common_free.clone()),
                                         only_b.values().map(|x| (**x).clone()).collect()));
            } else if ! only_b.is_empty() {
                return Err(format!("Cannot unify {} and {}", a, b));
            }

            Ok(())
        }
        (&Ty::Rec(ref extends, ref props), &Ty::Union(ref opts)) => {
            let nopts = opts.iter().map(|opt| {
                // Unify the record with every option
                try!(unify(scope, opt, &Ty::Rec(extends.clone(), props.clone())));

                // Get this option back into standard form
                let std_opt = std_form(scope, opt.clone());

                // Remove any duplicate properties
                if let Ty::Rec(ref oextends, ref oprops) = std_opt {
                    Ok(Ty::Rec(
                        oextends.clone(),
                        oprops.iter().filter_map(|x| {
                            if props.iter().all(|y| y.symbol() != x.symbol()) {
                                Some(x.clone())
                            } else { None }
                        }).collect()))
                } else {
                    panic!("Invariant Exception: after unifying with record, not record");
                }
            }).collect();

            if let Some(ref extends) = *extends {
                scope.substitute(
                    extends.unwrap_ident(),
                    Ty::Union(try!(nopts)));
                Ok(())
            } else {
                Err(format!("Fuuuuuuu"))
            }
        }
        (&Ty::Union(_), &Ty::Rec(_, _)) => {
            // This simply delegates to the above branch.
            // It doesn't do it in the most efficient way, but that is OK
            unify(scope, &b, &a)
        }
        (&Ty::Union(ref aopts), &Ty::Union(ref bopts)) => {
            // We need to find the intersection of the unions. kinda.
            // Its complicated, and I'm probably going to do it wrong.
            // Also, this will probably cause unacceptable growth of data
            // structures, which sucks.
            unimplemented!()
        }
    }
}
