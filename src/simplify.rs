use std::collections::HashMap;
use il::*;
use infer::InferValue;

pub fn simplify(iv: &InferValue) -> InferValue {
    let mut type_vars = HashMap::new();
    for (id, type_var) in iv.type_vars.iter() {
        let type_var = inline_type(type_var, &iv.type_vars);

        type_vars.insert(id.clone(), type_var);
    }
    
    let mut data_vars = HashMap::new();
    for (id, data_var) in iv.data_vars.iter() {
        let data_var = inline_type(data_var, &type_vars);

        data_vars.insert(id.clone(), data_var);
    }
    
    prune_tyvars(&InferValue {
        data_vars: data_vars,
        type_vars: type_vars,
    })
}

fn inline_type(ty: &Ty, type_vars: &HashMap<Ident, Ty>) -> Ty {
    let mut inlined = ty.clone();
    
    // Inline any identifiers or record extends
    loop {
        let mut new_value;
        match inlined {
            Ty::Ident(ref ident) => {
                // Don't inline non-internal identifiers
                if let Ident(_, Internal(_)) = *ident {
                    if let Some(ref ty) = type_vars.get(ident) {
                        new_value = (*ty).clone();
                    } else {
                        break;
                    }
                } else {
                    break;
                }

            }
            Ty::Rec(ref extends, ref values) => {
                match **extends {
                    Some(Ty::Ident(ref ident)) => {
                        if let Some(ref ty) = type_vars.get(ident) {
                            new_value = Ty::Rec(box Some((*ty).clone()), values.clone());
                        } else {
                            break;
                        }
                    }
                    Some(Ty::Rec(ref extends2, ref values2)) => {
                        let mut new_values = values.iter().chain(values2.iter()).map(|x| x.clone());
                        new_value = Ty::Rec(extends2.clone(), new_values.collect());
                    }
                    _ => {
                        break;
                    }
                }
            }
            _ => break
        }
        inlined = new_value;
    }
    
    inlined
}


fn prune_tyvars(iv: &InferValue) -> InferValue {
    fn copy(from: &HashMap<Ident, Ty>, to: &mut HashMap<Ident, Ty>, key: &Ident) {
        if let Some(ref v) = from.get(key) {
            if to.insert(key.clone(), (*v).clone()).is_none() {
                handle(from, to, *v);
            }
        }
    }

    fn handle(old_type_vars: &HashMap<Ident, Ty>, type_vars: &mut HashMap<Ident, Ty>, ty: &Ty) {
        match *ty {
            Ty::Ident(ref ident) => {
                copy(old_type_vars, type_vars, ident);
            }
            Ty::Rec(box ref extends, ref props) => {
                if let Some(ref ty) = *extends {
                    handle(old_type_vars, type_vars, ty);
                }
                for prop in props.iter() {
                    match *prop {
                        TyProp::Val(_, ref val) => {
                            handle(old_type_vars, type_vars, val);
                        }
                        TyProp::Method(_, ref params, ref result) => {
                            handle(old_type_vars, type_vars, result);
                            for param in params.iter() {
                                handle(old_type_vars, type_vars, param);
                            }
                        }
                    }
                }
            }
            Ty::Fn(ref params, box ref result) => {
                handle(old_type_vars, type_vars, result);
                for param in params.iter() {
                    handle(old_type_vars, type_vars, param);
                }
            }
        }
    }

    let mut type_vars = HashMap::new();
    for (_, data_var) in iv.data_vars.iter() {
        handle(&iv.type_vars, &mut type_vars, data_var);
    }
    
    InferValue {
        data_vars: iv.data_vars.clone(),
        type_vars: type_vars,
    }
}
