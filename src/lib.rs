use std::fmt::Debug;
pub use std::array::IntoIter;
pub use std::iter::FromIterator;
pub use std::collections::HashMap;

pub use std::fs;
pub use serde_json::json;
pub use std::error::Error;

mod worker;
mod operation;
mod recurse;
mod reconcile;

pub use serde_json::{Value as SerdeValue};
pub use reconcile::{Reconciler, WorkerReconciler};
pub use crate::operation::{Operation, OpType};


pub fn debug_print<T1, T2> (v1: T1, v2: T2, p: &str) where T1: Debug, T2: Debug {
    println!("Old Value Is: {:?}", v1);
    println!("New Value Is: {:?}", v2);
    println!("Path is: {:?}", p);
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
        let old_str = fs::read_to_string("./stubs/objects/update/old.json").unwrap_or(String::from("{}"));
        let old_str = old_str.as_str();
        let new_str = fs::read_to_string("./stubs/objects/update/new.json").unwrap_or(String::from("{}"));
        let new_str = new_str.as_str();

        let from: SerdeValue = serde_json::from_str(old_str).unwrap();
        let to: SerdeValue = serde_json::from_str(new_str).unwrap();

        let mut reconciler = Reconciler::new(&from, &to);
        // let worker = Box::from(WorkerMock{});

        let tree = reconciler.reconcile();
        let assertion = HashMap::from_iter(IntoIter::new([
            ("/a".to_owned(), Operation{ op: OpType::Update, from: Some(json!("a")), to: Some(json!("a-what?")) }),
            ("/arr/0/arr3/arrObj1".to_owned(), Operation{ op: OpType::Update, from: Some(json!("arrObj1")), to: Some(json!("arrObj2")) }),
            ("/arr/5".to_owned(), Operation{ op: OpType::Update, from: Some(json!(0.4)), to: Some(json!(0.3)) }),
        ]));

        assert_eq!(tree, assertion);
    }

    #[test]
    fn test_delete() {
        let old_str = fs::read_to_string("./stubs/objects/delete/old.json").unwrap_or(String::from("{}"));
        let old_str = old_str.as_str();
        let new_str = fs::read_to_string("./stubs/objects/delete/new.json").unwrap_or(String::from("{}"));
        let new_str = new_str.as_str();

        let from: SerdeValue = serde_json::from_str(old_str).unwrap();
        let to: SerdeValue = serde_json::from_str(new_str).unwrap();

        let mut reconciler = Reconciler::new(&from, &to);
        // let worker = Box::from(WorkerMock{});

        let tree = reconciler.reconcile();
        let assertion = HashMap::<_, _>::from_iter(IntoIter::new([
            ("/arr/0/arr1".to_owned(), Operation { op: OpType::Delete, from: Some(json!("arr1")), to: None }),
            ("/f".to_owned(), Operation { op: OpType::Delete, from: Some(json!(null)), to: None }),
            ("/arr/2".to_owned(), Operation { op: OpType::Delete, from: Some(json!("arr4")), to: None }),
            ("/arr/4".to_owned(), Operation { op: OpType::Delete, from: Some(json!(0.2)), to: None }),
            ("/e".to_owned(), Operation { op: OpType::Delete, from: Some(json!("e")), to: None }),
            ("/g".to_owned(), Operation { op: OpType::Delete, from: Some(json!(null)), to: None }),
            ("/arr/3".to_owned(), Operation { op: OpType::Delete, from: Some(json!(0.1)), to: None }),
            ("/obj/a3/3".to_owned(), Operation { op: OpType::Delete, from: Some(json!(4)), to: None }),
            ("/obj/a3/2".to_owned(), Operation { op: OpType::Delete, from: Some(json!(3)), to: None }),
            ("/obj/a3/4".to_owned(), Operation { op: OpType::Delete, from: Some(json!(0.5)), to: None }),
            ("/obj/a3/0".to_owned(), Operation { op: OpType::Delete, from: Some(json!(1)), to: None }),
            ("/c/2".to_owned(), Operation { op: OpType::Delete, from: Some(json!(3)), to: None }),
            ("/b".to_owned(), Operation { op: OpType::Delete, from: Some(json!("b")), to: None }),
            ("/d".to_owned(), Operation { op: OpType::Delete, from: Some(json!(132)), to: None }),
            ("/obj/a2".to_owned(), Operation { op: OpType::Delete, from: Some(json!("a2")), to: None }),
            ("/arr/0/arr3/arrObj1".to_owned(), Operation { op: OpType::Delete, from: Some(json!("arrObj1")), to: None }),
            ("/arr/5".to_owned(), Operation { op: OpType::Delete, from: Some(json!(0.3)), to: None }),
            ("/c/1".to_owned(), Operation { op: OpType::Delete, from: Some(json!("2")), to: None }),
            ("/obj/a3/1".to_owned(), Operation { op: OpType::Delete, from: Some(json!(2)), to: None }),
            ("/obj/a1".to_owned(), Operation { op: OpType::Delete, from: Some(json!("a1")), to: None }),
            ("/c/0".to_owned(), Operation { op: OpType::Delete, from: Some(json!("1")), to: None }),
            ("/arr/0/arr2".to_owned(), Operation { op: OpType::Delete, from: Some(json!("arr2")), to: None }),
            ("/arr/1/abc123".to_owned(), Operation { op: OpType::Delete, from: Some(json!("abc123")), to: None }),
            ("/a".to_owned(), Operation { op: OpType::Update, from: Some(serde_json::json!("a")), to: Some(json!("a-what?")) }),
        ]));

        assert_eq!(tree, assertion);
    }

    #[test]
    fn test_create() {
        let old_str = fs::read_to_string("./stubs/objects/create/old.json").unwrap_or(String::from("{}"));
        let old_str = old_str.as_str();
        let new_str = fs::read_to_string("./stubs/objects/create/new.json").unwrap_or(String::from("{}"));
        let new_str = new_str.as_str();

        let from: SerdeValue = serde_json::from_str(old_str).unwrap();
        let to: SerdeValue = serde_json::from_str(new_str).unwrap();

        let mut reconciler = Reconciler::new(&from, &to);
        // let worker = Box::from(WorkerMock{});

        let tree = reconciler.reconcile();
        let assertion = HashMap::<_, _>::from_iter(IntoIter::new([
            ("/abc".to_owned(), Operation { op: OpType::Create, from: None, to: Some(json!({
                "abc": {
                    "abc": "123",
                    "123": 123
                }
            }))}),
            ("/b".to_owned(), Operation { op: OpType::Create, from: None, to: Some(json!("b")) }),
            ("/abc123".to_owned(), Operation { op: OpType::Create, from: None, to: Some(json!("abc123")) }),
            ("/a".to_owned(), Operation { op: OpType::Update, from: Some(json!("a")), to: Some(json!("a-what?")) }),
            ("/array".to_owned(), Operation {op: OpType::Create, from: None, to: Some(json!([1, 2, 3]))}),
            ("/~1abc~1abc~1123~0~0~01".to_owned(), Operation {op: OpType::Create, from: None, to: Some(json!("/abc/abc/123~~~1")) }),
            ("/~0abc~0".to_owned(), Operation {op: OpType::Create, from: None, to: Some(json!("~abc~")) })
        ]));

        assert_eq!(tree, assertion);
    }
}
