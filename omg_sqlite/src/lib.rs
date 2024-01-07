use std::{error::Error, path::Path, sync::Arc};

use omg_core::{Message, Storage};
use serde_json::Value;
use sqlite::{Connection, ConnectionThreadSafe, State};
use time::OffsetDateTime;

pub fn file_blocking(path: impl AsRef<Path>) -> Result<Box<dyn Storage>, Box<dyn Error>> {
    let db = Connection::open_thread_safe(path)?;
    db.execute("CREATE TABLE IF NOT EXISTS messages (topic TEXT, seq INTEGER, created INTEGER, stored INTEGER, data TEXT)")?;
    Ok(Box::new(SqliteBackend { db }))
}

struct SqliteBackend {
    db: ConnectionThreadSafe,
}

impl Storage for SqliteBackend {
    fn append_blocking(
        &self,
        topic: &str,
        created: OffsetDateTime,
        data: Value,
    ) -> Result<(), Box<dyn Error>> {
        let mut statement = self
            .db
            .prepare("SELECT seq FROM messages WHERE topic = ? ORDER BY seq DESC LIMIT 1")?;
        statement.bind((1, topic))?;
        let seq = if statement.next()? == State::Row {
            statement.read("seq")?
        } else {
            0
        } + 1;

        let mut statement = self
            .db
            .prepare("INSERT INTO messages VALUES (:topic, :seq, :created, :stored, :data)")?;
        statement.bind((":topic", topic))?;
        statement.bind((":seq", seq))?;
        statement.bind((":created", created.unix_timestamp()))?;
        let stored = OffsetDateTime::now_utc();
        statement.bind((":stored", stored.unix_timestamp()))?;
        statement.bind((":data", data.to_string().as_str()))?;

        while statement.next()? != State::Done {}
        Ok(())
    }

    fn read_all_blocking(&self, topic: &str) -> Result<Vec<omg_core::Message>, Box<dyn Error>> {
        let mut statement = self.db.prepare(
            "SELECT seq, created, stored, data FROM messages WHERE topic = ? ORDER BY seq ASC",
        )?;
        statement.bind((1, topic))?;

        let topic_name: Arc<str> = Arc::from(topic);

        statement
            .into_iter()
            .map(|row| {
                let row = row?;

                Ok(Message {
                    topic_name: topic_name.clone(),
                    seq: row.try_read::<i64, _>("seq")? as u64,
                    created: OffsetDateTime::from_unix_timestamp(row.try_read("created")?)?,
                    stored: OffsetDateTime::from_unix_timestamp(row.try_read("stored")?)?,
                    data: serde_json::from_str(row.try_read("data")?)?,
                })
            })
            .collect()
    }
}
