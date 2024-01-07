use std::{error::Error, sync::Arc};

use serde_json::Value;
use time::OffsetDateTime;

pub struct Message {
    pub topic_name: Arc<str>,
    pub seq: u64,
    pub created: OffsetDateTime,
    pub stored: OffsetDateTime,
    pub data: Value,
}

pub trait Storage: Send + Sync {
    fn append_blocking(
        &self,
        topic: &str,
        created: OffsetDateTime,
        data: Value,
    ) -> Result<(), Box<dyn Error>>;
    fn read_all_blocking(&self, topic: &str) -> Result<Vec<Message>, Box<dyn Error>>;
}

pub struct Agency {
    storage: Arc<dyn Storage>,
}

impl Agency {
    pub fn new(storage: Box<dyn Storage>) -> Self {
        Agency {
            storage: storage.into(),
        }
    }

    pub fn storage(&self) -> &dyn Storage {
        self.storage.as_ref()
    }

    pub fn topic(&self, name: &str) -> Topic {
        Topic { name: name.to_owned(), storage: self.storage.clone() }
    }
}

pub struct Topic {
    name: String,
    storage: Arc<dyn Storage>,
}

impl Topic {
    pub fn publish(&self, data: Value) -> Result<(), Box<dyn Error>> {
        self.storage.append_blocking(&self.name, OffsetDateTime::now_utc(), data)
    } 
}
