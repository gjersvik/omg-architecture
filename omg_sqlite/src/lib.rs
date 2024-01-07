use std::{error::Error, path::Path, sync::Arc};

use omg_core::{Storage, StorageItem, StorageTopic};
use serde_json::Value;
use sqlite::{Connection, ConnectionThreadSafe, State};

pub fn file_blocking(path: impl AsRef<Path>) -> Result<Box<dyn Storage>, Box<dyn Error>> {
    let db = Connection::open_thread_safe(path)?;
    db.execute("CREATE TABLE IF NOT EXISTS messages (topic TEXT, seq INTEGER, data TEXT)")?;
    Ok(Box::new(SqliteBackend { db }))
}

struct SqliteBackend {
    db: ConnectionThreadSafe,
}

impl Storage for SqliteBackend {
    fn append_blocking(
        &self,
        topic: &str,
        seq: u64,
        data: Value,
    ) -> Result<(), Box<dyn Error>> {
        let mut statement = self
            .db
            .prepare("INSERT INTO messages VALUES (:topic, :seq, :data)")?;
        statement.bind((":topic", topic))?;
        statement.bind((":seq", seq as i64))?;
        statement.bind((":data", data.to_string().as_str()))?;

        while statement.next()? != State::Done {}
        Ok(())
    }

    fn read_all_blocking(&self, topic: &str) -> Result<Vec<StorageItem>, Box<dyn Error>> {
        let mut statement = self
            .db
            .prepare("SELECT seq, data FROM messages WHERE topic = ? ORDER BY seq ASC")?;
        statement.bind((1, topic))?;

        statement
            .into_iter()
            .map(|row| {
                let row = row?;

                Ok(StorageItem {
                    seq: row.try_read::<i64, _>("seq")? as u64,
                    data: serde_json::from_str(row.try_read("data")?)?,
                })
            })
            .collect()
    }

    fn topics(&self) -> Result<Vec<StorageTopic>, Box<dyn Error>> {
        let statement = self.db.prepare(
            "SELECT topic, min(seq) as first, max(seq) as last FROM messages GROUP BY topic",
        )?;
        statement
            .into_iter()
            .map(|row| {
                let row = row?;

                Ok(StorageTopic {
                    name: Arc::from(row.try_read::<&str, _>("topic")?),
                    first: row.try_read::<i64, _>("first")? as u64,
                    last: row.try_read::<i64, _>("last")? as u64,
                })
            })
            .collect()
    }
}
