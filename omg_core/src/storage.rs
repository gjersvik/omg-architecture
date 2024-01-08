use std::error::Error;

use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

pub struct StorageItem {
    pub seq: u64,
    pub data: Arc<str>,
}

pub struct StorageTopic {
    pub name: Arc<str>,
    pub first: u64,
    pub last: u64,
}

pub enum StorageEvent {
    Topics(oneshot::Sender<Result<Vec<StorageTopic>, Box<dyn Error + Send + Sync + 'static>>>),
    Push(
        Arc<str>,
        u64,
        Arc<str>,
        oneshot::Sender<Result<(), Box<dyn Error + Send + Sync + 'static>>>,
    ),
    ReadAll(
        Arc<str>,
        oneshot::Sender<Result<Vec<StorageItem>, Box<dyn Error + Send + Sync + 'static>>>,
    ),
}

pub type StoragePort = UnboundedSender<StorageEvent>;

pub(crate) trait Storage: Send + Sync {
    fn topics(&self) -> Result<Vec<StorageTopic>, Box<dyn Error>>;
    fn append_blocking(&self, topic: &str, seq: u64, data: &str) -> Result<(), Box<dyn Error>>;
    fn read_all_blocking(&self, topic: &str) -> Result<Vec<StorageItem>, Box<dyn Error>>;
}

impl Storage for StoragePort {
    fn append_blocking(&self, topic: &str, seq: u64, data: &str) -> Result<(), Box<dyn Error>> {
        let (send, recv) = oneshot::channel();
        self.send(StorageEvent::Push(topic.into(), seq, data.into(), send))
            .expect("Database thread is just gone");
        recv.blocking_recv()
            .expect("Database thread is just gone")
            .map_err(|e| e as Box<dyn Error>)
    }

    fn read_all_blocking(&self, topic: &str) -> Result<Vec<StorageItem>, Box<dyn Error>> {
        let (send, recv) = oneshot::channel();
        self.send(StorageEvent::ReadAll(topic.into(), send))
            .expect("Database thread is just gone");
        recv.blocking_recv()
            .expect("Database thread is just gone")
            .map_err(|e| e as Box<dyn Error>)
    }

    fn topics(&self) -> Result<Vec<StorageTopic>, Box<dyn Error>> {
        let (send, recv) = oneshot::channel();
        self.send(StorageEvent::Topics(send))
            .expect("Database thread is just gone");
        recv.blocking_recv()
            .expect("Database thread is just gone")
            .map_err(|e| e as Box<dyn Error>)
    }
}
