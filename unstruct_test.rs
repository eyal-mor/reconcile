use unstructured::{Document};
use serde::{Deserialize};
use std::collections::HashMap;

pub trait Recurse {
    fn recurse() {
        // TODO
        println!("I'm here!!")
    }
}

impl Recurse for Document {
    fn recurse() {
        println!("Did I recurse?")
    }
}

pub fn test_str() -> &'static str {
    return "test";
}

pub fn unstruct() {
    let something: Document = serde_json::from_str(r#"{"some": {"nested": {"vals": [1,2,3]}}}"#).unwrap();
    println!("{:?}", unstructured::walk!(something/"some"));
    Document
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn it_works2() {
        assert_eq!(test_str(), "test");
    }

    #[test]
    fn test_unstructured() {
        // use std::any::Any;
        use unstructured::Document;

        let something: Document = serde_json::from_str(r#"{"some": {"nested": {"vals": [1,2,3]}}}"#).unwrap();
        println!("{:?}", something);

        assert_eq!(true, true);
    }

    #[test]
    fn test_unstruct() {
        unstruct();
        assert_eq!(true, true);
    }
}
