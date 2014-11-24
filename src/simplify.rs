use std::collections::HashMap;
use il::*;
use infer::InferValue;

pub fn simplify(iv: &InferValue) -> InferValue {
    let mut type_vars = HashMap::new();

    for (id, type_var) in iv.type_vars.iter() {
        let mut type_var = type_var.clone();

        loop {
            let mut new_value;
            match type_var {
                Ty::Ident(ref ident) => {
                    if let Some(ref ty) = iv.type_vars.get(ident) {
                        new_value = (*ty).clone();
                    } else {
                        break;
                    }
                }
                Ty::Rec(ref extends, ref values) => {
                    match **extends {
                        Some(Ty::Ident(ref ident)) => {
                            if let Some(ref ty) = iv.type_vars.get(ident) {
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
            type_var = new_value;
        }

        type_vars.insert(id.clone(), type_var);
    }
    
    let mut data_vars = HashMap::new();
    for (id, data_var) in iv.data_vars.iter() {
        let mut data_var = data_var.clone();

        loop {
            let mut new_value;
            match data_var {
                Ty::Ident(ref ident) => {
                    if let Some(ref ty) = type_vars.get(ident) {
                        new_value = (*ty).clone();
                    } else {
                        break;
                    }
                }
                _ => break
            }
            data_var = new_value;
        }

        data_vars.insert(id.clone(), data_var);
    }
    
    prune_tyvars(&InferValue {
        data_vars: data_vars,
        type_vars: type_vars,
    })
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
