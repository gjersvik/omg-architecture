use std::{error::Error, sync::Arc};

use serde::{Serialize, Deserialize};
use serde_json::Value;
use time::OffsetDateTime;

pub struct StorageObj {
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
    fn read_all_blocking(&self, topic: &str) -> Result<Vec<StorageObj>, Box<dyn Error>>;
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

    pub fn topic(&self, name: &str) -> Topic {
        Topic { name: name.to_owned(), storage: self.storage.clone() }
    }
}

pub trait Message: Serialize + for<'a> Deserialize<'a> {
    
}

impl<T> Message for T where T: Serialize + for<'a> Deserialize<'a> {}

pub struct Topic {
    name: String,
    storage: Arc<dyn Storage>,
}

impl Topic {
    pub fn publish<M: Message>(&self, data: M) -> Result<(), Box<dyn Error>> {
        self.storage.append_blocking(&self.name, OffsetDateTime::now_utc(), serde_json::to_value(data)?)
    }

    pub fn subscribe<M: Message>(&self) -> Result<impl Iterator<Item = M>, Box<dyn Error>> {
        let vec = self.storage.read_all_blocking(&self.name)?;
        let data = vec.into_iter().map(|msg| serde_json::from_value::<M>(msg.data)).collect::<Result<Vec<M>, _>>()?;
        Ok(data.into_iter())
    }
}
