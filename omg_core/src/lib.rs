use std::{collections::HashMap, error::Error, marker::PhantomData, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

pub struct StorageObj {
    pub topic_name: Arc<str>,
    pub seq: u64,
    pub created: OffsetDateTime,
    pub stored: OffsetDateTime,
    pub data: Value,
}

pub struct StorageTopic {
    pub name: Arc<str>,
    pub first: u64,
    pub last: u64,
}

pub trait Storage: Send + Sync {
    fn topics(&self) -> Result<Vec<StorageTopic>, Box<dyn Error>>;
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
    topics: HashMap<Arc<str>, StorageTopic>,
}

impl Agency {
    pub fn load(storage: Box<dyn Storage>) -> Result<Self, Box<dyn Error>> {
        let topics = storage
            .topics()?
            .into_iter()
            .map(|t| (t.name.clone(), t))
            .collect();

        Ok(Agency {
            storage: storage.into(),
            topics,
        })
    }

    pub fn create_topic<M: Message>(&mut self, name: &str) -> Topic<M> {
        match self.topics.get(name) {
            Some(s) => Topic::new(s, self.storage.clone()),
            None => {
                let topic = StorageTopic {
                    name: Arc::from(name),
                    first: 0,
                    last: 0,
                };
                self.topics.insert(topic.name.clone(), topic);
                self.create_topic(name)
            }
        }
    }
}

pub trait Message: Serialize + for<'a> Deserialize<'a> {}

impl<T> Message for T where T: Serialize + for<'a> Deserialize<'a> {}

pub struct Topic<M: Message> {
    name: Arc<str>,
    _first: u64,
    _last: u64,
    storage: Arc<dyn Storage>,
    _marker: PhantomData<M>,
}

impl<M: Message> Topic<M> {
    fn new(state: &StorageTopic, storage: Arc<dyn Storage>) -> Self{
        Topic { name: state.name.clone(), _first: state.first, _last: state.last, storage, _marker: PhantomData }
    }

    pub fn publish(&self, data: M) -> Result<(), Box<dyn Error>> {
        self.storage.append_blocking(
            &self.name,
            OffsetDateTime::now_utc(),
            serde_json::to_value(data)?,
        )
    }

    pub fn subscribe(&self) -> Result<impl Iterator<Item = M>, Box<dyn Error>> {
        let vec = self.storage.read_all_blocking(&self.name)?;
        let data = vec
            .into_iter()
            .map(|msg| serde_json::from_value::<M>(msg.data))
            .collect::<Result<Vec<M>, _>>()?;
        Ok(data.into_iter())
    }
}
