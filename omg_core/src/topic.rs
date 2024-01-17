use std::{
    error::Error,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, watch};

use crate::{StorageEvent, StoragePort, StorageTopic};

pub trait Message: Serialize + for<'a> Deserialize<'a> {}

impl<T> Message for T where T: Serialize + for<'a> Deserialize<'a> {}

pub struct Topic<M: Message> {
    core: Arc<TopicCore>,
    _marker: PhantomData<M>,
}

impl<M: Message> Topic<M> {
    pub fn new(core: Arc<TopicCore>) -> Self {
        Topic {
            core,
            _marker: PhantomData,
        }
    }

    pub fn publish(&self, data: M) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.core.publish(serde_json::to_string(&data)?)?;
        Ok(())
    }
}

pub struct TopicCore {
    name: Arc<str>,
    last: watch::Sender<u64>,
    storage: StoragePort,
    atomic_publish: Mutex<()>,
}

impl TopicCore {
    pub fn new(topic: StorageTopic, storage: StoragePort) -> Arc<TopicCore> {
        let topic_core = TopicCore {
            name: topic.name,
            last: watch::Sender::new(topic.last),
            storage,
            atomic_publish: Mutex::new(()),
        };
        Arc::new(topic_core)
    }

    pub fn empty(name: Arc<str>, storage: StoragePort) -> Arc<TopicCore> {
        let topic_core = TopicCore {
            name,
            last: watch::Sender::new(0),
            storage,
            atomic_publish: Mutex::new(()),
        };
        Arc::new(topic_core)
    }

    pub fn publish(&self, data: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        let _stay_until_after_return = self.atomic_publish.lock().unwrap();

        let next = *self.last.borrow() + 1;
        let data: Arc<str> = data.into();
        // Save to disk
        let (send, recv) = oneshot::channel();
        self.storage.send(StorageEvent::Push(
            self.name.clone(),
            next,
            data.clone(),
            send,
        ))?;
        recv.blocking_recv()??;
        // Update last.
        self.last.send_replace(next);
        Ok(())
    }
}
