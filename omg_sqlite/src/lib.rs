use std::{error::Error, path::Path};

use omg_core::Storage;
use serde_json::Value;
use sqlite::{Connection, ConnectionThreadSafe};
use time::OffsetDateTime;

pub fn file_blocking(path: impl AsRef<Path>) -> Result<Box<dyn Storage>, Box<dyn Error>> {
    let db = Connection::open_thread_safe(path)?;
    db.execute("CREATE TABLE IF NOT EXISTS messages (topic TEXT, seq, INTEGER, created INTEGER, stored INTEGER, data TEXT)")?;
    Ok(Box::new(SqliteBackend { _db: db }))
}

struct SqliteBackend {
    _db: ConnectionThreadSafe,
}

impl Storage for SqliteBackend {
    fn append_blocking(
        &self,
        _topic: &str,
        _created: Option<OffsetDateTime>,
        _data: Value,
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn read_all_blocking(&self, _topic: &str) -> Result<Vec<omg_core::Message>, Box<dyn Error>> {
        todo!()
    }
}
