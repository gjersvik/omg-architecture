use std::sync::Arc;

pub trait Storage: Send + Sync {}

pub struct Agency {
    _storage: Option<Arc<dyn Storage>>,
}

impl Agency {
    pub fn new(storage: Option<Box<dyn Storage>>) -> Agency {
        Agency {
            _storage: storage.map(Into::into),
        }
    }

    pub fn load_blocking(&self, _name: &str) -> Agent {
        Agent {}
    }
}

pub struct Agent {}
