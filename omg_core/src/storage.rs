use std::error::Error;

use std::sync::Arc;

use thiserror::Error;
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

pub enum StorageInput {
    Topics(oneshot::Sender<Result<Vec<StorageTopic>, StorageError>>),
    Push(
        Arc<str>,
        u64,
        Arc<str>,
        oneshot::Sender<Result<(), StorageError>>,
    ),
    ReadAll(
        Arc<str>,
        oneshot::Sender<Result<Vec<StorageItem>, StorageError>>,
    ),
}

#[derive(Error, Debug, Clone)]
#[error(transparent)]
pub struct StorageError(Arc<dyn Error + Send + Sync>);

impl From<Box<dyn Error + Send + Sync>> for StorageError {
    fn from(value: Box<dyn Error + Send + Sync>) -> Self {
        StorageError(value.into())
    }
}
