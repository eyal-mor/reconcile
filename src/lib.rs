// Ideas:
//  1. Split path and search up the tree for a parent worker if no worker was found for child node.
//  2. Register by "schema", which is derived by keys/values and their types.
// mod observer;

use serde_json::{Value as SerdeValue, json};
use std::string::String;
use std::error::Error;
use std::fs;
use std::fmt::Debug;
use path_tree::PathTree;

pub fn debug_print<T1, T2> (v1: T1, v2: T2, p: &str) where T1: Debug, T2: Debug {
    println!("Old Value Is: {:?}", v1);
    println!("New Value Is: {:?}", v2);
    println!("Path is: {:?}", p);
}

pub trait Worker<'a> {
    fn create(&self, old_data: &SerdeValue, new_data: &SerdeValue, path: &str) -> Result<SerdeValue, Box<dyn Error>>;
    fn update(&self) -> Result<SerdeValue, Box<dyn Error>>;
    fn delete(&self, old_data: &SerdeValue, path: &str) -> Result<SerdeValue, Box<dyn Error>>;
    fn error_create(&self) {}
    fn error_update(&self) {}
    fn error_delete(&self) {}
}

pub struct Reconciler <'a> {
    old: SerdeValue,
    new: SerdeValue,
    observers: PathTree<Box<dyn Worker<'a>>>,
}

impl <'a> Reconciler <'a> {
    pub fn new(old: String, new: String) -> Result<Reconciler<'a>, serde_json::Error> {
        let old = match serde_json::from_str(&old.to_owned()) {
            Ok(v) => v,
            Err(e) => {
                println!("{:?}", e);
                return Err(e);
            }
        };

        let new = match serde_json::from_str(&new.to_owned()) {
            Ok(v) => v,
            Err(e) => {
                println!("{:?}", e);
                return Err(e);
            }
        };

        let observers = PathTree::new();
        Ok(Reconciler{old, new, observers,})
    }

    pub fn add_observer(&mut self, path: &'a str, observer: Box<dyn Worker<'a>>) {
        self.observers.insert(path, observer);
    }

    pub fn reconcile(&self) {
        self.recurse(&self.old, "");
    }

    fn recurse(&self, elem: &SerdeValue, p: &str) {
        match elem {
            SerdeValue::Null => {
                let new_data = match self.new.pointer(p) {
                    Some(d) => d,
                    None => {
                        match self.observers.find(p) {
                            Some(v) => {
                                let (worker, _) = v;
                                worker.delete(elem, p);
                            },
                            None => {
                                debug_print(elem, "NONE", p);
                            }
                        };

                        return;
                    }
                };

                if new_data.is_null() {
                    return;
                }

                match self.observers.find(p) {
                    Some(v) => {
                        let (worker, _) = v;
                        worker.create(elem, new_data, p);
                    },
                    None => {
                        debug_print(false, new_data, p);
                    },
                };
            },
            SerdeValue::Bool(old_data) => {
                let new_data = self.new.pointer(p).unwrap();

                if old_data == new_data {
                    return;
                }

                match self.observers.find(p) {
                    Some(v) => {
                        let (worker, _) = v;
                        worker.create(elem, new_data, p);
                    },
                    None => {
                        debug_print(false, new_data, p);
                    },
                };
            },
            SerdeValue::Number(old_data) => {
                let new_data = self.new.pointer(p).unwrap();
                if new_data.is_number() && old_data.as_f64().unwrap() == new_data.as_f64().unwrap() {
                    return;
                }

                match self.observers.find(p) {
                    Some(v) => {
                        let (worker, _) = v;
                        worker.create(elem, new_data, p);
                    },
                    None => {
                        debug_print(false, new_data, p);
                    },
                };
            },
            SerdeValue::String(old_data) => {
                let new_data = self.new.pointer(p).unwrap();
                if new_data.is_string() && old_data.as_str() == new_data.as_str().unwrap() {
                    return;
                }

                match self.observers.find(p) {
                    Some(v) => {
                        let (worker, _) = v;
                        worker.create(elem, new_data, p);
                    },
                    None => {
                        debug_print(false, new_data, p);
                    },
                };
            },
            SerdeValue::Array(old_data) => {
                for (pos, elem) in old_data.iter().enumerate() {
                    let new_p = format!("{}/{}", p, pos);
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

    #[derive(Debug, Clone)]
    pub struct WorkerMock {}

    impl <'a> Worker <'a> for WorkerMock {
        fn create(&self, old_data: &SerdeValue, new_data: &SerdeValue, p: &str) -> Result<SerdeValue, Box<dyn Error>>{
            println!("Called from WorkerMock::create");
            debug_print(old_data, new_data, p);
            Ok(json!(1))
        }
        fn update(&self) -> Result<SerdeValue, Box<dyn Error>>{
            return Ok(json!(1));
        }
        fn delete(&self, old_data: &SerdeValue, p: &str) -> Result<SerdeValue, Box<dyn Error>>{
            println!("Called from WorkerMock::delete");
            debug_print(old_data, "", p);
            Ok(json!(1))
        }
    }

    #[test]
    fn test_modify() {
        let old = fs::read_to_string("./stubs/simple-json/old.json").expect("this should have worked?");
        let new = fs::read_to_string("./stubs/simple-json/new.json").expect("this should have worked?");

        let mut reconciler = Reconciler::new(String::from(old), String::from(new)).unwrap();
        let worker = Box::from(WorkerMock{});
        reconciler.add_observer("/arr/*/arr3/arrObj1", worker.clone());
        reconciler.add_observer("/a", worker.clone());
        reconciler.reconcile();
    }

    #[test]
    fn test_delete() {
        let old = fs::read_to_string("./stubs/missing-new-values/old.json").expect("this should have worked?");
        let new = fs::read_to_string("./stubs/missing-new-values/new.json").expect("this should have worked?");

        let mut reconciler = Reconciler::new(String::from(old), String::from(new)).unwrap();
        let worker = Box::from(WorkerMock{});
        reconciler.add_observer("/arr/*/arr3/arrObj1", worker.clone());
        reconciler.add_observer("/a", worker.clone());
        reconciler.reconcile();
    }
}
