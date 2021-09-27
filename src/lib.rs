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
use recurse::{recurse, Changes, Direction};

pub fn debug_print<T1, T2> (v1: T1, v2: T2, p: &str) where T1: Debug, T2: Debug {
    println!("Old Value Is: {:?}", v1);
    println!("New Value Is: {:?}", v2);
    println!("Path is: {:?}", p);
}

trait Reconcile {
    fn reconcile(&mut self) -> Changes;
}

pub struct Reconciler <'a> {
    from: &'a SerdeValue,
    to: &'a SerdeValue,
}

impl <'a> Reconciler <'a> {
    pub fn new(from: &'a SerdeValue, to: &'a SerdeValue) -> Reconciler<'a> {
        Reconciler{from, to}
    }

    pub fn reconcile(&mut self) -> Changes {
        let mut changes = Changes::new();
        recurse(&self.from, &self.to, "", &mut changes, Direction::OldToNew);
        recurse(&self.to, &self.from, "", &mut changes, Direction::NewToOld);

        return changes;
    }
}

pub struct WorkerReconciler <'a> {
    from: &'a SerdeValue,
    to: &'a SerdeValue,
    observers: PathTree<Box<dyn worker::Worker<'a>>>,
    operations: HashMap<String, Operation>,
}

impl <'a> WorkerReconciler <'a> {
    pub fn new(from: &'a SerdeValue, to: &'a SerdeValue) -> WorkerReconciler<'a> {
        WorkerReconciler{
            from,
            to,
            observers: PathTree::new(),
            operations: HashMap::new(),
        }
    }

    pub fn add_observer(&mut self, path: &str, observer: Box<dyn worker::Worker<'a>>) {
        self.observers.insert(path, observer);
    }

    pub fn reconcile(&mut self) -> Changes {
        let mut changes = Changes::new();
        recurse(&self.from, &self.to, "", &mut changes, Direction::OldToNew);
        recurse(&self.to, &self.from, "", &mut changes, Direction::NewToOld);
        self.call_observers_on_changes(&changes);

        return changes;
    }

    fn call_observers_on_changes(&mut self, changes: &Changes) {
        for (path, op) in changes.into_iter() {
            let path = path.as_str();
            match self.observers.find(path) {
                Some(v) => {
                    let (worker, _) = v;

                    match op.op {
                        OpType::Create => {
                            worker.create(op.to.as_ref().unwrap(), path);
                        },
                        OpType::Update => {
                            worker.update(op.from.as_ref().unwrap(), op.to.as_ref().unwrap(), path);
                        },
                        OpType::Delete => {
                            worker.delete(op.from.as_ref().unwrap(), path);
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
        // let worker = Box::from(WorkerMock{});

        let tree = reconciler.reconcile();
        println!("tree is: {:?}", tree);
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
        // let worker = Box::from(WorkerMock{});

        let tree = reconciler.reconcile();
        println!("tree is: {:?}", tree);
    }

    #[test]
    fn test_create() {
        let old_str = fs::read_to_string("./stubs/test-create/old.json").unwrap_or(String::from("{}"));
        let old_str = old_str.as_str();
        let new_str = fs::read_to_string("./stubs/test-create/new.json").unwrap_or(String::from("{}"));
        let new_str = new_str.as_str();

        let from: SerdeValue = serde_json::from_str(old_str).unwrap();
        let to: SerdeValue = serde_json::from_str(new_str).unwrap();

        let mut reconciler = Reconciler::new(&from, &to);
        // let worker = Box::from(WorkerMock{});

        let tree = reconciler.reconcile();
        println!("tree is: {:?}", tree);
    }
}
