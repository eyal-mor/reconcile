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
    old: &'a SerdeValue,
    new: &'a SerdeValue,
    observers: PathTree<Box<dyn worker::Worker<'a>>>,
    // Hashmap with: k = path (aka pointer into path), v = tuple(operation, old_data, new_data)
    operations: HashMap<String, Operation>,
}

impl <'a> Reconciler <'a> {
    pub fn new(old: &'a SerdeValue, new: &'a SerdeValue) -> Reconciler<'a> {
        Reconciler{
            old,
            new,
            observers: PathTree::new(),
            operations: HashMap::new(),
        }
    }

    pub fn add_observer(&mut self, path: &str, observer: Box<dyn worker::Worker<'a>>) {
        self.observers.insert(path, observer);
    }

    pub fn reconcile(&mut self) {
        self.recurse(&self.old, "");
    }

    pub fn add_operation(&mut self, path: &str, operation: Operation) {
        self.operations.insert(path.to_owned(), operation);
    }

    // match self.observers.find(p) {
    //     Some(v) => {
    //         let (worker, _) = v;
    //         worker.update(elem, new_data, p);
    //     },
    //     None => {
    //         debug_print(false, new_data, p);
    //     },
    // };

    fn recurse(&mut self, elem: &'a SerdeValue, p: &str) {
        match elem {
            SerdeValue::Null => {
                let new_data = match self.new.pointer(p) {
                    Some(d) => d,
                    None => {
                        let operation = Operation{
                            op: OpType::Delete,
                            new: None,
                            old: Option::from(elem.clone()),
                        };

                        self.add_operation(p, operation);
                        return;
                    }
                };

                if new_data.is_null() {
                    return;
                }

                let operation = Operation{
                    op: OpType::Update,
                    new: Option::from(elem.clone()),
                    old: Option::from(new_data.to_owned()),
                };

                self.add_operation(p, operation);
            },
            SerdeValue::Bool(old_data) => {
                let new_data = match self.new.pointer(p) {
                    Some(d) => d,
                    None => {
                        let operation = Operation{
                            op: OpType::Delete,
                            new: None,
                            old: Option::from(elem.clone()),
                        };

                        self.add_operation(p, operation);
                        return;
                    }
                };

                if old_data == new_data {
                    return;
                }

                let operation = Operation {
                    op: OpType::Update,
                    new: Option::from(new_data.to_owned()),
                    old: Option::from(elem.to_owned()),
                };

                self.add_operation(p, operation);
            },
            SerdeValue::Number(old_data) => {
                let new_data = match self.new.pointer(p) {
                    Some(d) => d,
                    None => {
                        let operation = Operation{
                            op: OpType::Delete,
                            new: None,
                            old: Option::from(elem.clone()),
                        };

                        self.add_operation(p, operation);
                        return;
                    }
                };

                if new_data.is_number() && old_data.as_f64().unwrap() == new_data.as_f64().unwrap() {
                    return;
                }

                let operation = Operation {
                    op: OpType::Update,
                    new: Option::from(new_data.to_owned()),
                    old: Option::from(elem.to_owned()),
                };

                self.add_operation(p, operation);
            },
            SerdeValue::String(old_data) => {
                let new_data = match self.new.pointer(p) {
                    Some(d) => d,
                    None => {
                        let operation = Operation{
                            op: OpType::Delete,
                            new: None,
                            old: Option::from(elem.clone()),
                        };

                        self.add_operation(p, operation);
                        return;
                    }
                };

                if new_data.is_string() && old_data.as_str() == new_data.as_str().unwrap() {
                    return;
                }

                let operation = Operation {
                    op: OpType::Update,
                    new: Option::from(new_data.to_owned()),
                    old: Option::from(elem.to_owned()),
                };

                self.add_operation(p, operation);
            },
            SerdeValue::Array(old_data) => {
                for (pos, elem) in old_data.iter().enumerate() {
                    let new_p = Box::new(format!("{}/{}", p, pos));
                    self.recurse(elem, &new_p);
                }
            },
            SerdeValue::Object(old_data) => {
                for k in old_data.keys() {
                    match old_data.get(k) {
                        Some(v) => {
                            let new_p = format!("{}/{}", p, k);
                            self.recurse(v, &new_p);
                        },
                        None => {
                            println!("Skipped! {:?}", k);
                        }
                    }
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
        fn create(&self, old_data: &SerdeValue, new_data: &SerdeValue, p: &str) -> Result<SerdeValue, Box<dyn Error>>{
            println!("Called from WorkerMock::create");
            debug_print("", new_data, p);
            Ok(json!(1))
        }
        fn update(&self, old_data: &SerdeValue, new_data: &SerdeValue, p: &str) -> Result<SerdeValue, Box<dyn Error>>{
            println!("Called from WorkerMock::update");
            debug_print(old_data, new_data, p);
            Ok(json!(1))
        }
        fn delete(&self, old_data: &SerdeValue, p: &str) -> Result<SerdeValue, Box<dyn Error>>{
            println!("Called from WorkerMock::delete");
            debug_print(old_data, "", p);
            Ok(json!(1))
        }
    }

    #[test]
    fn test_modify() {
        let old_str = fs::read_to_string("./stubs/test-update/old.json").unwrap_or(String::from("{}"));
        let old_str = old_str.as_str();
        let new_str = fs::read_to_string("./stubs/test-update/new.json").unwrap_or(String::from("{}"));
        let new_str = new_str.as_str();

        let old: SerdeValue = serde_json::from_str(old_str).unwrap();
        let new: SerdeValue = serde_json::from_str(new_str).unwrap();

        let mut reconciler = Reconciler::new(&old, &new);
        let worker = Box::from(WorkerMock{});

        reconciler.add_observer("/arr/*/arr3/arrObj1", worker.clone());
        reconciler.add_observer("/a", worker.clone());
        reconciler.reconcile();
        println!("Test update: {:?}", reconciler.operations);
    }

    #[test]
    fn test_delete() {
        let old_str = fs::read_to_string("./stubs/test-delete/old.json").unwrap_or(String::from("{}"));
        let old_str = old_str.as_str();
        let new_str = fs::read_to_string("./stubs/test-delete/new.json").unwrap_or(String::from("{}"));
        let new_str = new_str.as_str();

        let old: SerdeValue = serde_json::from_str(old_str).unwrap();
        let new: SerdeValue = serde_json::from_str(new_str).unwrap();

        let mut reconciler = Reconciler::new(&old, &new);
        let worker = Box::from(WorkerMock{});

        reconciler.add_observer("/arr/*/arr3/arrObj1", worker.clone());
        reconciler.add_observer("/a", worker.clone());
        reconciler.reconcile();
        println!("Test Delete: {:?}", reconciler.operations);
    }
}
