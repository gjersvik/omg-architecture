mod agent;
mod channel;
mod storage;
mod topic;

use std::{collections::BTreeMap, error::Error, sync::Arc};

pub use agent::*;
pub use channel::*;
pub use storage::*;
use tokio::sync::oneshot;
pub use topic::*;

pub struct Agency {
    storage: StoragePort,
    topics: BTreeMap<Arc<str>, Arc<TopicCore>>,
}

impl Agency {
    pub fn load(storage: StoragePort) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut topics = BTreeMap::new();
        let (send, recv) = oneshot::channel();
        storage.send(StorageEvent::Topics(send))?;
        for topic in recv.blocking_recv()??.into_iter() {
            let (send, recv) = oneshot::channel();
            storage.send(StorageEvent::ReadAll(topic.name.clone(), send))?;
            let data = recv.blocking_recv()??;
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
