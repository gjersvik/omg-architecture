use std::{sync::Arc, collections::BTreeMap};

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

    pub fn load_blocking<K,V>(&self, _name: &str) -> Agent<K,V> {
        Agent { _view: BTreeMap::new() }
    }
}

pub struct Agent<K,V> {
    _view: BTreeMap<K,V>
}

