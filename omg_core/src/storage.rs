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
    Topics(oneshot::Sender<Result<Vec<StorageTopic>, Box<dyn Error + Send + Sync>>>),
    Push(
        Arc<str>,
        u64,
        Arc<str>,
        oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    ),
    ReadAll(
        Arc<str>,
        oneshot::Sender<Result<Vec<StorageItem>, Box<dyn Error + Send + Sync>>>,
    ),
}

pub type StoragePort = UnboundedSender<StorageEvent>;

pub(crate) trait Storage: Send + Sync {
    fn append_blocking(
        &self,
        topic: &str,
        seq: u64,
        data: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn read_all_blocking(
        &self,
        topic: &str,
    ) -> Result<Vec<StorageItem>, Box<dyn Error + Send + Sync>>;
}

impl Storage for StoragePort {
    fn append_blocking(
        &self,
        topic: &str,
        seq: u64,
        data: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (send, recv) = oneshot::channel();
        self.send(StorageEvent::Push(topic.into(), seq, data.into(), send))?;
        recv.blocking_recv()?
    }

    fn read_all_blocking(
        &self,
        topic: &str,
    ) -> Result<Vec<StorageItem>, Box<dyn Error + Send + Sync>> {
        let (send, recv) = oneshot::channel();
        self.send(StorageEvent::ReadAll(topic.into(), send))?;
        recv.blocking_recv()?
    }
}
