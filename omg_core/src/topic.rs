use std::{
    collections::BTreeMap,
    error::Error,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, watch};

use crate::{Receiver, Sender, StorageEvent, StorageItem, StoragePort, StorageTopic};

pub trait Message: Serialize + for<'a> Deserialize<'a> {}

impl<T> Message for T where T: Serialize + for<'a> Deserialize<'a> {}

pub struct Topic<M: Message> {
    core: Arc<TopicCore>,
    _marker: PhantomData<M>,
}

impl<M: Message> Topic<M> {
    pub(crate) fn new(core: Arc<TopicCore>) -> Self {
        Topic {
            core,
            _marker: PhantomData,
        }
    }

    pub fn publish(&self, data: M) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.core.publish(serde_json::to_string(&data)?)?;
        Ok(())
    }

    pub fn subscribe(&self) -> impl Iterator<Item = Result<M, Box<dyn Error + Send + Sync>>> {
        Subscribe::new(self.core.clone())
    }
}

impl<M: Message + Clone> Sender for Topic<M> {
    type Item = M;

    fn send(&self, value: Self::Item) {
        self.publish(value).unwrap()
    }
}

pub struct Subscribe<M: Message> {
    core: Arc<TopicCore>,
    current: u64,
    _marker: PhantomData<M>,
}

impl<M: Message> Subscribe<M> {
    fn new(core: Arc<TopicCore>) -> Self {
        let current = core.first();
        Subscribe {
            core,
            current,
            _marker: PhantomData,
        }
    }
}

impl<M: Message> Iterator for Subscribe<M> {
    type Item = Result<M, Box<dyn Error + Send + Sync>>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.core.get(self.current);
        if value.is_some() {
            self.current += 1;
        }
        value.map(|v| Ok(serde_json::from_str::<M>(&v)?))
    }
}

impl<M: Message + Clone + Send> Receiver for Subscribe<M> {
    type Item = M;

    fn recv(&mut self) -> Option<Self::Item> {
        self.next().map(|r| r.unwrap())
    }

    async fn async_recv(&mut self) -> Option<Self::Item> {
        todo!()
    }

    fn try_recv(&mut self) -> Result<Self::Item, crate::TryError> {
        todo!()
    }
}

pub(crate) struct TopicCore {
    name: Arc<str>,
    first: watch::Sender<u64>,
    last: watch::Sender<u64>,
    storage: StoragePort,
    cache: Mutex<BTreeMap<u64, Arc<str>>>,
    atomic_publish: Mutex<()>,
}

impl TopicCore {
    pub fn new(
        topic: StorageTopic,
        data: Vec<StorageItem>,
        storage: StoragePort,
    ) -> Arc<TopicCore> {
        let topic_core = TopicCore {
            name: topic.name,
            first: watch::Sender::new(topic.first),
            last: watch::Sender::new(topic.last),
            storage,
            cache: Mutex::new(data.into_iter().map(|item| (item.seq, item.data)).collect()),
            atomic_publish: Mutex::new(()),
        };
        Arc::new(topic_core)
    }

    pub fn empty(name: Arc<str>, storage: StoragePort) -> Arc<TopicCore> {
        let topic_core = TopicCore {
            name,
            first: watch::Sender::new(1),
            last: watch::Sender::new(0),
            storage,
            cache: Mutex::new(BTreeMap::new()),
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
        // Save to cache
        {
            self.cache.lock().unwrap().insert(next, data);
        }
        // Update last.
        self.last.send_replace(next);
        Ok(())
    }

    pub fn get(&self, seq: u64) -> Option<Arc<str>> {
        self.cache.lock().unwrap().get(&seq).cloned()
    }

    pub fn first(&self) -> u64 {
        *self.first.borrow()
    }
}
