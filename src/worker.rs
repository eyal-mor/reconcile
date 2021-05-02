use serde_json::{Value as SerdeValue};
use std::error::Error;

pub trait Worker<'a> {
    fn create(&self, old_data: &SerdeValue, new_data: &SerdeValue, path: &str) -> Result<SerdeValue, Box<dyn Error>>;
    fn update(&self, old_data: &SerdeValue, new_data: &SerdeValue, path: &str) -> Result<SerdeValue, Box<dyn Error>>;
    fn delete(&self, old_data: &SerdeValue, path: &str) -> Result<SerdeValue, Box<dyn Error>>;
    fn error_create(&self) {}
    fn error_update(&self) {}
    fn error_delete(&self) {}
}