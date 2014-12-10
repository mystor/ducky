use std::vec::Vec;
use std::collections::{HashMap, HashSet};
use infer::env::Environment;
use il::*;

struct Stage<'a> {
    env: &'a mut Environment,
    subs: HashMap<Ident, Ty>,
    unified: HashSet<(Ty, Ty)>,
}

impl <'a> Stage<'a> {
    fn new<'a>(env: &'a mut Environment) -> Stage<'a> {
        Stage{
            env: env,
            subs: HashMap::new(),
            unified: HashSet::new(),
        }
    }

    fn child(&'a mut self) -> Stage<'a> {
        Stage{
            env: self.env,
            subs: self.subs.clone(),
            unified: self.unified.clone(),
        }
    }

    fn apply(self) {
        for (id, ty) in self.subs.into_iter() {
            self.env.substitute(id.clone(), ty.clone());
        }
    }

    fn substitute(&mut self, a: Ident, b: Ty) {
        self.subs.insert(a, b);
    }

    fn lookup_type_var(&self, id: &Ident) -> Option<&Ty> {
        self.subs.get(id).or_else(|| self.env.lookup_type_var(id))
    }

    fn introduce_type_var(&mut self) -> Ty {
        self.env.introduce_type_var()
    }
}

fn unify_props<'a>(stage: &mut Stage<'a>, a: &TyProp, b: &TyProp) -> Result<(), String> {
    match (a, b) {
        (&TyProp::Val(_, ref aty), &TyProp::Val(_, ref bty)) => {
            _unify(stage, aty, bty)
        }
        (&TyProp::Method(_, ref aargs, ref ares), &TyProp::Method(_, ref bargs, ref bres)) => {
            if aargs.len() != bargs.len() {
                return Err(format!("Cannot unify {} and {}", a, b));
            }

            // Unify each of the arguments
            for (aarg, barg) in aargs.iter().zip(bargs.iter()) {
                try!(_unify(stage, aarg, barg));
            }

            _unify(stage, ares, bres)
        }
        _ => {
            Err(format!("Cannot unify properties: {} and {}", a, b))
        }
    }
}

/// Reduces the type to it's "standard form". In this form, all
/// "root" Ty::Idents are not bound in the stage, and there are no
/// nested unions.
fn std_form<'a>(stage: &Stage<'a>, ty: Ty) -> Ty {
    match ty {
        Ty::Ident(ref id) => {
            if let Some(nty) = stage.lookup_type_var(id) {
                std_form(stage, nty.clone())
            } else { ty.clone() }
        }
        Ty::Rec(ref extends, ref props) => {
            if let Some(box ref extends) = *extends {
                let nextends = std_form(stage, extends.clone());

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
                            std_form(stage,
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
                std_form(stage, opt.clone())
            }).collect();
            // TODO: Expand inner unions!

            Ty::Union(opts)
        }
    }
}

fn _unify<'a>(stage: &mut Stage<'a>, a: &Ty, b: &Ty) -> Result<(), String> {
    let ty_pairs = (a.clone(), b.clone());
    if stage.unified.contains(&(a.clone(), b.clone())) {
        return Ok(());
    } else {
        // If they haven't been unified before, assume that they have!
        stage.unified.insert(ty_pairs);
    }

    // Types in this language are very simple, they all take the form of records, or
    // unions of records. Which is going to be nice for us.
    // We need to first reduce both type a and type b to standard form, and then
    // unify them in standard form.
    let a = std_form(stage, a.clone());
    let b = std_form(stage, b.clone());

    match (&a, &b) {
        (&Ty::Ident(ref a), b) => {
            stage.substitute(a.clone(), b.clone());

            Ok(())
        }
        (a, &Ty::Ident(ref b)) => {
            stage.substitute(b.clone(), a.clone());

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
                try!(unify_props(stage, aprop, bprop));
            }

            let common_free = stage.introduce_type_var();

            if let Some(box Ty::Ident(ref ident)) = *bextends {
                // We need to unify bextends with something
                stage.substitute(ident.clone(),
                                 Ty::Rec(Some(box common_free.clone()),
                                         only_a.values().map(|x| (**x).clone()).collect()));
            } else if ! only_a.is_empty() {
                return Err(format!("Cannot unify {} and {}", a, b));
            }

            // Merge the remaining values into the other maps
            if let Some(box Ty::Ident(ref ident)) = *aextends {
                // We need to unify bextends with something
                stage.substitute(ident.clone(),
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
                try!(_unify(stage, opt, &Ty::Rec(extends.clone(), props.clone())));

                // Get this option back into standard form
                let std_opt = std_form(stage, opt.clone());

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
                stage.substitute(
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
            _unify(stage, &b, &a)
        }
        (&Ty::Union(ref aopts), &Ty::Union(ref bopts)) => {
            // We need to find the intersection of the unions. kinda.
            // Its complicated, and I'm probably going to do it wrong.
            // Also, this will probably cause unacceptable growth of data
            // structures, which sucks.

            // First, we need to determine what the final union will look like
            // The final union will have stuff and things
            // it will be awesome
            // yeah
            // woop!

            struct UniOpt<'a> {
                aopt: &'a Ty,
                bopt: &'a Ty,
                tyvar: Ty,
            }

            impl <'a> UniOpt<'a> {
                fn flip(&self) -> UniOpt<'a> {
                    UniOpt{
                        aopt: self.bopt,
                        bopt: self.aopt,
                        tyvar: self.tyvar.clone()
                    }
                }
            }

            let mut uniopts = Vec::new();

            for aopt in aopts.iter() {
                for bopt in bopts.iter() {
                    // Check if these two can unify! We do this in a child
                    // scope such that no modifications are made to the
                    // actual global scope! (Very fancy!)
                    // Rather, this only checks if the unification is possible.

                    // It shouldn't loop infinitely (I think), as it (kinda)
                    // shares the same Unified hashmap.
                    if _unify(&mut stage.child(), aopt, bopt).is_ok() {
                        uniopts.push(UniOpt{
                            aopt: aopt,
                            bopt: bopt,
                            tyvar: stage.introduce_type_var()
                        });
                    }
                }
            }

            fn something<'a>(stage: &mut Stage<'a>, aopt: &Ty, uniopts: &Vec<UniOpt<'a>>) -> Result<(), String> {
                let filtered = uniopts.iter().filter(|x| x.aopt == aopt);

                // unify the objects together, woo!
                try!(_unify(stage, aopt,
                            &Ty::Union(filtered.map(|x| x.bopt.clone()).collect())));

                // We need to monkey-patch the extension variable if it exists to the
                // one which we want from the uniopts

                // These first two conditionals are just checking invariants. We need
                // to unwrap some data structures, and may as well make sure that
                // some invariants are held along the way!
                if let Ty::Rec(Some(box Ty::Ident(ref extends)), _) = *aopt {
                    let union = stage.lookup_type_var(extends).unwrap().clone();
                    if let Ty::Union(ref opts) = union {
                        // Finally, we have reached the opts which were created by
                        // the unification (hopefully?)

                        // We need to pair each opt with its uniopt
                        let mut zipped =
                            uniopts.iter().filter(|x| x.aopt == aopt).zip(opts.iter());
                        for (uniopt, opt) in zipped {
                            // Unify the uniopt's tyvar with the opt's extension,
                            // This ensures that the two sides will use the same tyvar
                            match *opt {
                                Ty::Rec(Some(box ref extends), _) => {
                                    try!(_unify(stage, extends, &uniopt.tyvar));
                                }
                                Ty::Rec(None, _) => {/* Can this happen? */}
                                Ty::Ident(_) => {
                                    try!(_unify(stage, aopt, opt));
                                }
                                _ => unreachable!("Invariant")
                            }
                        }
                    } else {
                        unreachable!("Invariant");
                    }
                } else {
                    unreachable!("Invariant");
                }
                Ok(())
            }

            for aopt in aopts.iter() {
                try!(something(stage, aopt, &uniopts));
            }

            for bopt in bopts.iter() {
                try!(something(stage, bopt, &uniopts.iter().map(|x| x.flip()).collect()));
            }

            Ok(())
        }
    }
}

pub fn unify<'a>(env: &mut Environment, a: &Ty, b: &Ty) -> Result<(), String> {
    let mut stage = Stage::new(env);
    try!(_unify(&mut stage, a, b));
    stage.apply();
    Ok(())
}
