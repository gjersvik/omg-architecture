mod storage;
mod topic;

use std::{collections::BTreeMap, error::Error, sync::Arc};

pub use storage::*;
pub use topic::*;

pub struct Agency {
    storage: StoragePort,
    topics: BTreeMap<Arc<str>, Arc<TopicCore>>,
}

impl Agency {
    pub fn load(storage: StoragePort) -> Result<Self, Box<dyn Error>> {
        let mut topics = BTreeMap::new();
        for topic in storage.topics()?.into_iter() {
            let data = storage.read_all_blocking(&topic.name)?;
            topics.insert(
                topic.name.clone(),
                TopicCore::new(topic, data, storage.clone()),
            );
        }

        Ok(Agency { storage, topics })
    }

    pub fn create_topic<M: Message>(&mut self, name: &str) -> Topic<M> {
        match self.topics.get(name) {
            Some(core) => Topic::new(core.clone()),
            None => {
                let name: Arc<str> = Arc::from(name);
                self.topics.insert(
                    name.clone(),
                    TopicCore::empty(name.clone(), self.storage.clone()),
                );
                self.create_topic(&name)
            }
        }
    }
}
