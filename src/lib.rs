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
mod recurse;

use operation::{Operation, OpType};
use recurse::{recurse, Changes};

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
        let mut changes = Changes::new();
        recurse(&self.from, &self.to, "", &mut changes);
        recurse(&self.to, &self.from, "", &mut changes);

        for (path, op) in changes.into_iter() {
            let path = path.as_str();
            match self.observers.find(path) {
                Some(v) => {
                    let (worker, _) = v;

                    match op.op {
                        OpType::Create => {
                            worker.create(&op.to.unwrap(), path);
                        },
                        OpType::Update => {
                            worker.update(&op.from.unwrap(), &op.to.unwrap(), path);
                        },
                        OpType::Delete => {
                            worker.delete(&op.from.unwrap(), path);
                        },
                    }
                },
                None => {
                    println!("No Operation Found For {:#?}: {:#?}", path, op);
                },
            };
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
        fn create(&self, new_data: &SerdeValue, p: &str) -> Result<SerdeValue, Box<dyn Error>>{
            println!("Called from WorkerMock::create");
            debug_print("", new_data, p);
            Ok(json!(1))
        }
        fn update(&self, old_data: &SerdeValue, new_data: &SerdeValue, p: &str) -> Result<SerdeValue, Box<dyn Error>>{
            println!("Called from WorkerMock::update");
            debug_print(old_data, new_data, p);
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
        // println!("Test update: {:#?}", changes);
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
        // println!("Test Delete: {:#?}", changes);
    }
}
