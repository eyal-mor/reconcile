use serde_json::{Value as SerdeValue};
use std::string::String;
use std::collections::HashMap;
use path_tree::PathTree;

use crate::operation::{Operation, OpType};
use crate::recurse::{recurse, Changes};
use crate::worker;

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
        recurse(&self.from, &self.to, "", &mut changes);

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
        recurse(&self.from, &self.to, "", &mut changes);
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
