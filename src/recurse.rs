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
                match from_data.get(k) {
                    Some(v) => {
                        let new_p = format!("{}/{}", p, k);
                        recurse(v, comp, &new_p, changes);
                    },
                    None => {
                        println!("Skipped! {:?}", k);
                    }
                };
            }
        },
    }
}