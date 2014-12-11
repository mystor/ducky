use std::collections::HashSet;
use il::*;
use infer::env::Env;

pub fn free_vars<'a>(stage: &mut (Env + 'a), ty: &Ty) -> HashSet<Ident> {
    _free_vars(stage, ty, &mut HashSet::new())
}

fn _free_vars<'a>(stage: &mut (Env + 'a), ty: &Ty, checked: &mut HashSet<Ty>) -> HashSet<Ident> {
    let mut idents = HashSet::new();
    if checked.contains(ty) { return idents } else { checked.insert(ty.clone()); }

    match *ty {
        Ty::Ident(ref id) => {
            if let Some(ty) = stage.lookup_type_var(id).cloned() {
                return _free_vars(stage, &ty.clone(), checked)
            } else {
                idents.insert(id.clone());
            }
        }
        Ty::Rec(ref extends, ref props) => {
            if let Some(box ref extends) = *extends {
                idents.extend(_free_vars(stage, extends, checked).iter().cloned());
            }

            for prop in props.iter() {
                match *prop {
                    TyProp::Val(_, ref ty) => {
                        idents.extend(_free_vars(stage, ty, checked).iter().cloned());
                    }
                    TyProp::Method(_, ref args, ref res) => {
                        for arg in args.iter() {
                            idents.extend(_free_vars(stage, arg, checked).iter().cloned());
                        }
                        idents.extend(_free_vars(stage, res, checked).iter().cloned());
                    }
                }
            }
        }
        Ty::Union(ref opts) => {
            for opt in opts.iter() {
                idents.extend(_free_vars(stage, opt, checked).iter().cloned());
            }
        }
    }

    idents
}

pub fn toplevel_vars<'a>(stage: &mut (Env + 'a), ty: &Ty) -> HashSet<Ident> {
    let mut idents = HashSet::new();

    match *ty {
        Ty::Ident(ref id) => {
            if let Some(ty) = stage.lookup_type_var(id).cloned() {
                return toplevel_vars(stage, &ty.clone())
            } else {
                idents.insert(id.clone());
            }
        }
        Ty::Rec(ref extends, _) => {
            if let Some(box ref extends) = *extends {
                idents.extend(toplevel_vars(stage, extends).iter().cloned());
            }
        }
        Ty::Union(ref opts) => {
            for opt in opts.iter() {
                idents.extend(toplevel_vars(stage, opt).iter().cloned());
            }
        }
    }

    idents
}

pub fn val_ty(scope: &mut Env, ty: Ty) -> Ty {
    Ty::Union(vec![ty, scope.introduce_type_var()])
}
