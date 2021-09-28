use serde_json::{Value as SerdeValue};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OpType {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    pub op: OpType,
    pub to: Option<SerdeValue>,
    pub from: Option<SerdeValue>,
}
