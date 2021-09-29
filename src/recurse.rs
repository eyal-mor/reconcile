use serde_json::{Value as SerdeValue};
use std::collections::HashMap;
use crate::operation::{Operation, OpType};

pub type Changes = HashMap<String, Operation>;

pub fn recurse<'a>(elem: &'a SerdeValue, comp: &'a SerdeValue, p: &str, changes: &mut Changes) {
    match elem {
        SerdeValue::Null => {
            let comp_data = match comp.pointer(p) {
                Some(d) => d,
                None => {
                    let operation = Operation{
                        op: OpType::Delete,
                        to: None,
                        from: Option::from(elem.clone()),
                    };

                    changes.insert(String::from(p), operation);
                    return;
                }
            };

            if comp_data.is_null() {
                return;
            }

            let operation = Operation{
                op: OpType::Update,
                to: Option::from(elem.clone()),
                from: Option::from(comp_data.to_owned()),
            };

            changes.insert(String::from(p), operation);
        },
        SerdeValue::Bool(from_data) => {
            let comp_data = match comp.pointer(p) {
                Some(d) => d,
                None => {
                    let operation = Operation{
                        op: OpType::Delete,
                        to: None,
                        from: Option::from(elem.clone()),
                    };

                    changes.insert(String::from(p), operation);
                    return;
                }
            };

            if from_data == comp_data {
                return;
            }

            let operation = Operation {
                op: OpType::Update,
                to: Option::from(comp_data.to_owned()),
                from: Option::from(elem.to_owned()),
            };

            changes.insert(String::from(p), operation);
        },
        SerdeValue::Number(from_data) => {
            let comp_data = match comp.pointer(p) {
                Some(d) => d,
                None => {
                    let operation = Operation{
                        op: OpType::Delete,
                        to: None,
                        from: Option::from(elem.clone()),
                    };

                    changes.insert(String::from(p), operation);
                    return;
                }
            };

            if comp_data.is_number() && from_data.as_f64().unwrap() == comp_data.as_f64().unwrap() {
                return;
            }

            let operation = Operation {
                op: OpType::Update,
                to: Option::from(comp_data.to_owned()),
                from: Option::from(elem.to_owned()),
            };

            changes.insert(String::from(p), operation);
        },
        SerdeValue::String(from_data) => {
            let comp_data = match comp.pointer(p) {
                Some(d) => d,
                None => {
                    let operation = Operation{
                        op: OpType::Delete,
                        to: None,
                        from: Option::from(elem.clone()),
                    };

                    changes.insert(String::from(p), operation);
                    return;
                }
            };

            if comp_data.is_string() && from_data.as_str() == comp_data.as_str().unwrap() {
                return;
            }

            let operation = Operation {
                op: OpType::Update,
                to: Option::from(comp_data.to_owned()),
                from: Option::from(elem.to_owned()),
            };

            changes.insert(String::from(p), operation);
        },
        SerdeValue::Array(from_data) => {
            for (pos, elem) in from_data.iter().enumerate() {
                let new_p = Box::new(format!("{}/{}", p, pos));
                recurse(elem, comp, &new_p, changes);
            }
        },
        SerdeValue::Object(from_data) => {
            for k in from_data.keys() {
                let key_pointer = k.replace("~", "~0").replace("/", "~1");
                let new_p = format!("{}/{}", p, key_pointer);
                match from_data.get(k) {
                    Some(v) => {
                        recurse(v, comp, &new_p, changes);
                    },
                    None => { /* Skipped */ }
                };
            }
        },
    }

    let comp_val = comp.pointer(p).unwrap_or_else(|| &SerdeValue::Null);
    match (elem, comp_val) {
        (SerdeValue::Object(from_data), SerdeValue::Object(to_data)) => {
            let new_keys: Vec<_> = to_data.keys().filter(|k| !from_data.contains_key(k.clone())).collect();

            new_keys.into_iter().for_each(|k| {
                let key_pointer = k.replace("~", "~0").replace("/", "~1");
                let new_p = format!("{}/{}", p, key_pointer);

                let operation = Operation {
                    op: OpType::Create,
                    to: Option::from(comp_val.get(k).unwrap().clone()),
                    from: Option::None,
                };

                changes.insert(String::from(&new_p), operation);
            });
        },
        _ => { /* Skipped */ }
    }
}