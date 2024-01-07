use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use tokio::sync::watch;

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
    topics: HashMap<Arc<str>, Arc<RwLock<TopicCore>>>
}

impl Agency {
    pub fn load(storage: Box<dyn Storage>) -> Result<Self, Box<dyn Error>> {
        let storage: Arc<dyn Storage> = storage.into();
        let mut topics = HashMap::new();
        for topic in storage.topics()?.into_iter() {
            let data = storage.read_all_blocking(&topic.name)?;
            topics.insert(topic.name.clone(), TopicCore::new(topic, data, storage.clone()));
        }

        Ok(Agency {
            storage,
            topics,
        })
    }

    pub fn create_topic<M: Message>(&mut self, name: &str) -> Topic<M> {
        match self.topics.get(name) {
            Some(core) => Topic::new(core.clone()),
            None => {
                let name: Arc<str> = Arc::from(name); 
                self.topics.insert(name.clone(), TopicCore::empty(name.clone(), self.storage.clone()));
                self.create_topic(&name)
            }
        }
    }
}

pub trait Message: Serialize + for<'a> Deserialize<'a> {}

impl<T> Message for T where T: Serialize + for<'a> Deserialize<'a> {}

pub struct Topic<M: Message> {
    core: Arc<RwLock<TopicCore>>,
    _marker: PhantomData<M>,
}

impl<M: Message> Topic<M> {
    fn new(core: Arc<RwLock<TopicCore>>) -> Self {
        Topic {
            core,
            _marker: PhantomData,
        }
    }

    pub fn publish(&self, data: M) -> Result<(), Box<dyn Error>> {
        self.core.write().unwrap().publish(serde_json::to_value(data)?)?;
        Ok(())
    }

    pub fn subscribe(&self) -> impl Iterator<Item = Result<M, Box<dyn Error>>>{
        Subscribe::new(self.core.clone())
    }
}

pub struct Subscribe<M: Message> {
    core: Arc<RwLock<TopicCore>>,
    current: u64,
    _marker: PhantomData<M>,
}

impl<M: Message> Subscribe<M> {
    fn new(core: Arc<RwLock<TopicCore>>) -> Self {
        let current = core.read().unwrap().first();
        Subscribe { core, current, _marker: PhantomData}
    }
}

impl<M: Message> Iterator for Subscribe<M> {
    type Item = Result<M, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.core.read().unwrap().get(self.current);
        if value.is_some(){
            self.current += 1;
        }
        value.map(|v|Ok(serde_json::from_value::<M>(v)?))
    }
}

struct TopicCore {
    name: Arc<str>,
    first: watch::Sender<u64>,
    last: watch::Sender<u64>,
    storage: Arc<dyn Storage>,
    cache: BTreeMap<u64, Value>,
}

impl TopicCore {
    pub fn new(topic: StorageTopic, data: Vec<StorageObj>, storage: Arc<dyn Storage>) -> Arc<RwLock<TopicCore>> {
        let topic_core = TopicCore {
            name: topic.name,
            first: watch::Sender::new(topic.first),
            last: watch::Sender::new(topic.last),
            storage,
            cache: data.into_iter().map(|item| (item.seq, item.data)).collect(),
        };
        Arc::new(RwLock::new(topic_core))
    }

    pub fn empty(name: Arc<str>, storage: Arc<dyn Storage>) -> Arc<RwLock<TopicCore>> {
        let topic_core = TopicCore {
            name,
            first: watch::Sender::new(1),
            last: watch::Sender::new(0),
            storage,
            cache: BTreeMap::new(),
        };
        Arc::new(RwLock::new(topic_core))
    }

    pub fn publish(&mut self, data: Value) -> Result<(), Box<dyn Error>>{
        let next =  *self.last.borrow() + 1;
        // Save to disk
        self.storage.append_blocking(&self.name, OffsetDateTime::now_utc(), data.clone())?;
        // Save to cache
        self.cache.insert(next, data);
        // Update last.
        self.last.send_replace(next);
        Ok(())
    }

    pub fn get(&self, seq: u64) -> Option<Value> {
        self.cache.get(&seq).cloned()
    }

    pub fn first(&self) -> u64 {
        *self.first.borrow()
    } 
}
