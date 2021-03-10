use serde_json::{Value};
use std::string::String;
use std::error::Error;

pub trait Worker {
    fn create(&self) -> Result<Value, Box<dyn Error>>;
    fn update(&self) -> Result<Value, Box<dyn Error>>;
    fn delete(&self) -> Result<Value, Box<dyn Error>>;
}

pub struct Reconciler {
    old: Value,
    new: Value,
    observers: Vec<Box<dyn Worker>>
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
        let observers = vec![];

        Ok(Reconciler{old, new, observers})
    }

    pub fn reconcile(&self) {
        let p = String::from("");
        self.recurse(&self.old, &p);
    }

    fn recurse(&self, elem: &Value, p: &String) {
        match elem {
            Value::Null => {
                let v2_data = self.new.pointer(&p.to_owned()[..]).unwrap();
                if v2_data.is_null() {
                    println!("--------------------------------------");
                    println!("v2_data is Null");
                    println!("--------------------------------------");
                }
            },
            Value::Bool(d) => {
                let v2_data = self.new.pointer(&p.to_owned()[..]).unwrap();
                if d != v2_data {
                    println!("--------------------------------------");
                    println!("d == v2_d : {}", d == v2_data);
                    println!("--------------------------------------");
                }
            },
            Value::Number(d) => {
                let v2_data = self.new.pointer(&p.to_owned()[..]).unwrap();
                if v2_data.is_number() && d.as_f64().unwrap() != v2_data.as_f64().unwrap() {
                    println!("--------------------------------------");
                    println!("d == v2_d : {}", d.as_f64().unwrap() == v2_data.as_f64().unwrap());
                    println!("--------------------------------------");
                }
            },
            Value::String(d) => {
                let v2_data = self.new.pointer(&p.to_owned()[..]).unwrap();
                if v2_data.is_string() && d.as_str() != v2_data.as_str().unwrap() {
                    println!("--------------------------------------");
                    println!("{} == {} : {} for path: {}", d.as_str(), v2_data.as_str().unwrap(), d.as_str() == v2_data.as_str().unwrap(), p);
                    println!("--------------------------------------");
                }
            },
            Value::Array(d) => {
                println!("--------------------------------------");
                for (pos, elem) in d.iter().enumerate() {
                    let new_p = format!("{}/{}", p, pos);
                    self.recurse(elem, &new_p);
                }
                println!("--------------------------------------");
            },
            Value::Object(d) => {
                println!("--------------------------------------");
                for k in d.keys() {
                    match d.get(k) {
                        Some(v) => {
                            let new_p = format!("{}/{}", p, k);
                            self.recurse(v, &new_p);
                        },
                        None => {
                            println!("Skipped! {:?}", k);
                        }
                    }
                }
                println!("--------------------------------------");
            },
        }
    }
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

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


        let reconciler = Reconciler::new(String::from(old), String::from(new)).unwrap();
        println!("Old object is {:?}", reconciler.old);
        reconciler.reconcile();
    }
}
