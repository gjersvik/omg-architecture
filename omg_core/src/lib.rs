use std::{error::Error, sync::Arc};

use serde_json::Value;
use time::OffsetDateTime;

pub struct Message {
    pub topic_name: Arc<str>,
    pub seq: u64,
    pub created: OffsetDateTime,
    pub stored: OffsetDateTime,
    pub data: Value,
}

pub trait Storage: Send + Sync {
    fn append_blocking(
        &self,
        topic: &str,
        created: Option<OffsetDateTime>,
        data: Value,
    ) -> Result<(), Box<dyn Error>>;
    fn read_all_blocking(&self, topic: &str) -> Result<Vec<Message>, Box<dyn Error>>;
}
