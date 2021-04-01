use serde_json::{Value as SerdeValue, json};
use std::string::String;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

pub fn debug_print<T1, T2> (v1: T1, v2: T2, p: &str) where T1: Debug, T2: Debug {
    println!("--------------------------------------");
    println!("Old Value Is: {:?}", v1);
    println!("New Value Is: {:?}", v2);
    println!("Path is: {:?}", p);
    println!("--------------------------------------");
}

pub trait Worker {
    fn create(&self) -> Result<SerdeValue, Box<dyn Error>>;
    fn update(&self) -> Result<SerdeValue, Box<dyn Error>>;
    fn delete(&self) -> Result<SerdeValue, Box<dyn Error>>;
}

pub struct Reconciler {
    old: SerdeValue,
    new: SerdeValue,
    observers: HashMap<String, Box<dyn Worker>>
}

impl Reconciler {
    pub fn new(old: String, new: String) -> Result<Reconciler, serde_json::Error> {
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

        let observers = HashMap::new();
        Ok(Reconciler{old, new, observers})
    }

    pub fn add_observer(&mut self, key: String, observer: Box<dyn Worker>) {
        self.observers.insert(key, observer);
    }

    pub fn reconcile(&self) {
        self.recurse(&self.old, "");
    }

    fn recurse(&self, elem: &SerdeValue, p: &str) {
        match elem {
            SerdeValue::Null => {
                let new_data = self.new.pointer(p).unwrap();
                if !new_data.is_null() {
                    return;
                }

                match self.observers.get(p) {
                    Some(v) => {
                        v.create();
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

                match self.observers.get(p) {
                    Some(v) => {
                        v.create();
                    },
                    None => {
                        debug_print(old_data, new_data, p);
                    }
                };
            },
            SerdeValue::Number(old_data) => {
                let new_data = self.new.pointer(p).unwrap();
                if new_data.is_number() && old_data.as_f64().unwrap() == new_data.as_f64().unwrap() {
                    return;
                }

                match self.observers.get(p) {
                    Some(v) => {
                        v.create();
                    },
                    None => {
                        debug_print(old_data, new_data, p);
                    }
                };
            },
            SerdeValue::String(old_data) => {
                let new_data = self.new.pointer(p).unwrap();
                if new_data.is_string() && old_data.as_str() == new_data.as_str().unwrap() {
                    return;
                }

                match self.observers.get(p) {
                    Some(v) => {
                        v.create();
                    },
                    None => {
                        debug_print(old_data, new_data, p);
                    }
                };
            },
            SerdeValue::Array(old_data) => {
                // println!("--------------------------------------");
                for (pos, elem) in old_data.iter().enumerate() {
                    let new_p = format!("{}/{}", p, pos);
                    self.recurse(elem, &new_p);
                }
                // println!("--------------------------------------");
            },
            SerdeValue::Object(old_data) => {
                // println!("--------------------------------------");
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

    pub struct WorkerMock {}

    impl Worker for WorkerMock {
        fn create(&self) -> Result<SerdeValue, Box<dyn Error>>{
            return Ok(json!(1));
        }
        fn update(&self) -> Result<SerdeValue, Box<dyn Error>>{
            return Ok(json!(1));
        }
        fn delete(&self) -> Result<SerdeValue, Box<dyn Error>>{
            return Ok(json!(1));
        }
    }

    #[test]
    fn it_works() {
        let old = r#"
            {
                "a": "a",
                "b": "b",
                "c": ["1", "2", 3],
                "d": 132,
                "e": "e",
                "obj": {
                    "a1": "a1",
                    "a2": "a2",
                    "a3": [1,2,3,4,0.5]
                },
                "f": null,
                "g": null,
                "arr": [{"arr1": "arr1", "arr2": "arr2", "arr3": {"arrObj1": "arrObj1"}}, {"abc123": "abc123"}, "arr4", 0.1, 0.2, 0.3]
            }
        "#;

        let new = r#"
            {
                "a": "a-what?",
                "b": "b",
                "c": ["1", "2", 3],
                "d": 132,
                "e": "e",
                "obj": {
                    "a1": "a1",
                    "a2": "a2",
                    "a3": [1,2,3,4,0.5]
                },
                "f": null,
                "g": null,
                "arr": [{"arr1": "arr1", "arr2": "arr2", "arr3": {"arrObj1": "arrObj2"}}, {"abc123": "abc123"}, "arr4", 0.1, 0.2, 0.3]
            }
        "#;


        let mut reconciler = Reconciler::new(String::from(old), String::from(new)).unwrap();
        reconciler.add_observer(String::from("/arr/0/arr3/arrObj1"), Box::from(WorkerMock{}));
        println!("Old object is {:?}", reconciler.old);

        reconciler.reconcile();
    }
}
