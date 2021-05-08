// Ideas:
//  1. Split path and search up the tree for a parent worker if no worker was found for child node.
//  2. Register by "schema", which is derived by keys/values and their types.
// mod observer;

use serde_json::{Value as SerdeValue};
use std::string::String;
use std::collections::HashMap;
use std::fmt::Debug;
use path_tree::PathTree;

pub use serde_json::json;
pub use std::fs;
pub use std::error::Error;

mod worker;
mod operation;

use operation::{Operation, OpType};

pub fn debug_print<T1, T2> (v1: T1, v2: T2, p: &str) where T1: Debug, T2: Debug {
    println!("Old Value Is: {:?}", v1);
    println!("New Value Is: {:?}", v2);
    println!("Path is: {:?}", p);
}

pub struct Reconciler <'a> {
    from: &'a SerdeValue,
    to: &'a SerdeValue,
    observers: PathTree<Box<dyn worker::Worker<'a>>>,
    operations: HashMap<String, Operation>,
}

impl <'a> Reconciler <'a> {
    pub fn new(from: &'a SerdeValue, to: &'a SerdeValue) -> Reconciler<'a> {
        Reconciler{
            from,
            to,
            observers: PathTree::new(),
            operations: HashMap::new(),
        }
    }

    pub fn add_observer(&mut self, path: &str, observer: Box<dyn worker::Worker<'a>>) {
        self.observers.insert(path, observer);
    }

    pub fn reconcile(&mut self) {
        self.recurse(&self.from, &self.to, "");
        self.recurse(&self.to, &self.from, "");
    }

    pub fn add_operation(&mut self, path: &str, operation: Operation) {
        self.operations.insert(path.to_owned(), operation);
    }

    // match self.observers.find(p) {
    //     Some(v) => {
    //         let (worker, _) = v;
    //         worker.update(elem, comp_data, p);
    //     },
    //     None => {
    //         debug_print(false, comp_data, p);
    //     },
    // };

    fn recurse(&mut self, elem: &'a SerdeValue, comp: &'a SerdeValue, p: &str) {
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

                        self.add_operation(p, operation);
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

                self.add_operation(p, operation);
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

                        self.add_operation(p, operation);
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

                self.add_operation(p, operation);
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

                        self.add_operation(p, operation);
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

                self.add_operation(p, operation);
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

                        self.add_operation(p, operation);
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

                self.add_operation(p, operation);
            },
            SerdeValue::Array(from_data) => {
                for (pos, elem) in from_data.iter().enumerate() {
                    let new_p = Box::new(format!("{}/{}", p, pos));
                    self.recurse(elem, comp, &new_p);
                }
            },
            SerdeValue::Object(from_data) => {
                for k in from_data.keys() {
                    match from_data.get(k) {
                        Some(v) => {
                            let new_p = format!("{}/{}", p, k);
                            self.recurse(v, comp, &new_p);
                        },
                        None => {
                            println!("Skipped! {:?}", k);
                        }
                    };
                }
            },
        }
    }
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[derive(Debug, Copy, Clone)]
    pub struct WorkerMock {}

    impl <'a> worker::Worker <'a> for WorkerMock {
        fn create(&self, from_data: &SerdeValue, comp_data: &SerdeValue, p: &str) -> Result<SerdeValue, Box<dyn Error>>{
            println!("Called from WorkerMock::create");
            debug_print("", comp_data, p);
            Ok(json!(1))
        }
        fn update(&self, from_data: &SerdeValue, comp_data: &SerdeValue, p: &str) -> Result<SerdeValue, Box<dyn Error>>{
            println!("Called from WorkerMock::update");
            debug_print(from_data, comp_data, p);
            Ok(json!(1))
        }
        fn delete(&self, from_data: &SerdeValue, p: &str) -> Result<SerdeValue, Box<dyn Error>>{
            println!("Called from WorkerMock::delete");
            debug_print(from_data, "", p);
            Ok(json!(1))
        }
    }

    #[test]
    fn test_modify() {
        let old_str = fs::read_to_string("./stubs/test-update/old.json").unwrap_or(String::from("{}"));
        let old_str = old_str.as_str();
        let new_str = fs::read_to_string("./stubs/test-update/new.json").unwrap_or(String::from("{}"));
        let new_str = new_str.as_str();

        let from: SerdeValue = serde_json::from_str(old_str).unwrap();
        let to: SerdeValue = serde_json::from_str(new_str).unwrap();

        let mut reconciler = Reconciler::new(&from, &to);
        let worker = Box::from(WorkerMock{});

        reconciler.add_observer("/arr/*/arr3/arrObj1", worker.clone());
        reconciler.add_observer("/a", worker.clone());
        reconciler.reconcile();
        println!("Test update: {:#?}", reconciler.operations);
    }

    #[test]
    fn test_delete() {
        let old_str = fs::read_to_string("./stubs/test-delete/old.json").unwrap_or(String::from("{}"));
        let old_str = old_str.as_str();
        let new_str = fs::read_to_string("./stubs/test-delete/new.json").unwrap_or(String::from("{}"));
        let new_str = new_str.as_str();

        let from: SerdeValue = serde_json::from_str(old_str).unwrap();
        let to: SerdeValue = serde_json::from_str(new_str).unwrap();

        let mut reconciler = Reconciler::new(&from, &to);
        let worker = Box::from(WorkerMock{});

        reconciler.add_observer("/arr/*/arr3/arrObj1", worker.clone());
        reconciler.add_observer("/a", worker.clone());
        reconciler.reconcile();
        println!("Test Delete: {:#?}", reconciler.operations);
    }
}
