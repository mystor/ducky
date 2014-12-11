use std::vec::Vec;
use std::collections::{HashMap, HashSet};
use infer::util::{free_vars, toplevel_vars};
use infer::env::Env;
use infer::InferValue;
use il::*;

/// A stage is an extension of a parsing environment. It wraps around
/// the internal environment, and acts as a staging ground for substitutions
/// The substitutions are applied when `fn apply(mut self)` is called.
struct Stage<'a> {
    env: &'a mut (Env + 'a),
    subs: HashMap<Ident, Ty>,
    unified: HashSet<(Ty, Ty)>,
}

impl <'a> Stage<'a> {
    fn new<'a>(env: &'a mut (Env + 'a)) -> Stage<'a> {
        Stage{
            env: env,
            subs: HashMap::new(),
            unified: HashSet::new(),
        }
    }

    fn apply(mut self) {
        for (id, ty) in self.subs.into_iter() {
            self.env.substitute(id.clone(), ty.clone());
        }
    }
}

impl <'a> Env for Stage<'a> {
    fn substitute(&mut self, a: Ident, b: Ty) {
        // Verify that the substitutions are non-recursive
        if toplevel_vars(self, &b).contains(&a) {
            panic!("Cannot perform a toplevel-recursive substitution");
        }

        self.subs.insert(a, b);
    }

    fn lookup_data_var(&mut self, id: &Ident) -> Ty {
        self.env.lookup_data_var(id)
    }

    fn lookup_type_var(&self, id: &Ident) -> Option<&Ty> {
        self.subs.get(id).or_else(|| self.env.lookup_type_var(id))
    }

    fn introduce_type_var(&mut self) -> Ty {
        self.env.introduce_type_var()
    }

    fn as_infervalue(&self) -> InferValue {
        let mut iv = self.env.as_infervalue();
        iv.type_vars.extend(self.subs.iter().map(|(x, y)| (x.clone(), y.clone())));
        iv
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
                        let opts: Vec<Ty> = opts.iter().map(|opt| {
                            std_form(stage,
                                     Ty::Rec(Some(box opt.clone()), props.clone()))
                        }).collect();

                        // None of these should be unions, so we don't have to
                        // expand any inner unions (let's make sure!)
                        assert!(opts.iter().all(|x: &Ty| ! x.is_union()));

                        Ty::Union(opts)
                    }
                }
            } else { ty.clone() }
        }
        Ty::Union(ref opts) => {
            let opts = opts.iter().map(|opt| {
                // Convert every property into standard form!
                std_form(stage, opt.clone())
            }).fold(Vec::new(), |mut vec, opt| {
                // Pull in any unions which exist while transforming into a Vec
                if let Ty::Union(ref opts) = opt {
                    vec.push_all(opts.as_slice());
                } else {
                    vec.push(opt);
                }

                vec
            });

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

            let common_free = if aextends.is_none() || bextends.is_none() {
                None
            } else {
                Some(box stage.introduce_type_var())
            };

            if let Some(box Ty::Ident(ref ident)) = *bextends {
                // We need to unify bextends with something
                stage.substitute(ident.clone(),
                                 Ty::Rec(common_free.clone(),
                                         only_a.values().map(|x| (**x).clone()).collect()));
            } else if ! only_a.is_empty() {
                return Err(format!("Cannot unify {} and {}", a, b));
            }

            // Merge the remaining values into the other maps
            if let Some(box Ty::Ident(ref ident)) = *aextends {
                // We need to unify bextends with something
                stage.substitute(ident.clone(),
                                 Ty::Rec(common_free.clone(),
                                         only_b.values().map(|x| (**x).clone()).collect()));
            } else if ! only_b.is_empty() {
                return Err(format!("Cannot unify {} and {}", a, b));
            }

            Ok(())
        }
        (&Ty::Rec(_, _), &Ty::Union(ref opts)) => {
            // We can't unify a concrete record with a union, so let's not even try!
            let mut subs: HashMap<Ident, Vec<Ty>> = HashMap::new();

            for opt in opts.iter() {
                // These are the free variables in opt
                // Actually whoops...
                let free = free_vars(stage, opt);

                // Unify with the option in a child_stage
                let mut child_stage = Stage::new(stage);

                try!(_unify(&mut child_stage, opt, &a));

                // Localsubs records all of the entries which are free in opt
                let mut localsubs = HashMap::new();
                for (id, ty) in child_stage.subs.iter() {
                    if ! free.contains(id) {
                        if let Some(lst) = subs.get_mut(id) {
                            lst.push(ty.clone());
                            continue
                        }

                        subs.insert(id.clone(), vec![ty.clone()]);
                    } else {
                        localsubs.insert(id.clone(), ty.clone());
                    }
                }

                // Modify the child_stage to have those values be free in opt
                child_stage.subs = localsubs;

                // TODO: We need to make this only apply up one level, rather than all of the way to the top
                // also, we need to not copy everything. Basically, I need to rewrite the way that stages work
                // It'll be fun!
                // Merge these subs up one level into stage!
                child_stage.apply();
            }

            for (id, tys) in subs.iter() {
                try!(_unify(stage, &Ty::Ident(id.clone()), &Ty::Union(tys.clone())));
            }

            Ok(())
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

            #[deriving(Show)]
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
                    if _unify(&mut Stage::new(stage), aopt, bopt).is_ok() {
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
                let extends = aopt.unwrap_extends().unwrap_ident();
                if let Some(ref union) = stage.lookup_type_var(&extends).cloned() {
                    if let Ty::Union(ref opts) = *union {
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
                    Ok(())
                } else {
                    Err("Can't unify, because we're missing stuff! woop! I'm not sure what went wrong, lets find out later".to_string())
                }
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

pub fn unify<'a>(env: &mut (Env + 'a), a: &Ty, b: &Ty) -> Result<(), String> {
    let mut stage = Stage::new(env);

    try!(_unify(&mut stage, a, b));

    stage.apply();
    Ok(())
}
