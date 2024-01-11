use std::error::Error;

use std::sync::Arc;

use thiserror::Error;
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
    Topics(oneshot::Sender<Result<Vec<StorageTopic>, StorageError>>),
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

#[derive(Error, Debug, Clone)]
#[error(transparent)]
pub struct StorageError(Arc<dyn Error + Send + Sync>);

impl From<Box<dyn Error + Send + Sync>> for StorageError{
    fn from(value: Box<dyn Error + Send + Sync>) -> Self {
        StorageError(value.into())
    }
}
