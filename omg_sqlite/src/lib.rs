use std::error::Error;

use omg_core::Storage;
use serde_json::Value;
use time::OffsetDateTime;

pub fn file(_path: &str) -> Result<Box<dyn Storage>, String> {
    Ok(Box::new(SqliteBackend{}))
}

struct SqliteBackend {

}

impl Storage for SqliteBackend {
    fn append_blocking(&self, _topic: &str, _created: Option<OffsetDateTime>, _data: &Value) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn read_all(&self, _topic: &str) -> Result<Vec<omg_core::Message>, Box<dyn Error>> {
        todo!()
    }
}