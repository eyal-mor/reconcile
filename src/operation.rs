use serde_json::{Value as SerdeValue};

#[derive(Debug, Copy, Clone)]
pub enum OpType {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub op: OpType,
    pub to: Option<SerdeValue>,
    pub from: Option<SerdeValue>,
}
