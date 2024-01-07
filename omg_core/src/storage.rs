use std::error::Error;

use std::sync::Arc;

pub struct StorageItem {
    pub seq: u64,
    pub data: Arc<str>,
}

pub struct StorageTopic {
    pub name: Arc<str>,
    pub first: u64,
    pub last: u64,
}

pub trait Storage: Send + Sync {
    fn topics(&self) -> Result<Vec<StorageTopic>, Box<dyn Error>>;
    fn append_blocking(&self, topic: &str, seq: u64, data: &str) -> Result<(), Box<dyn Error>>;
    fn read_all_blocking(&self, topic: &str) -> Result<Vec<StorageItem>, Box<dyn Error>>;
}
