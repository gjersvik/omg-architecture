use std::{collections::BTreeMap, sync::Arc};

pub trait Storage: Send + Sync {}
pub trait Key: Ord {}
impl<T: Ord> Key for T {}

pub struct Agency {
    _storage: Option<Arc<dyn Storage>>,
}

impl Agency {
    pub fn new(storage: Option<Box<dyn Storage>>) -> Agency {
        Agency {
            _storage: storage.map(Into::into),
        }
    }

    pub fn get<K: Key, V>(&self, _name: &str) -> Agent<K, V> {
        Agent {
            view: BTreeMap::new(),
        }
    }
}

pub struct Agent<K: Key, V> {
    view: BTreeMap<K, V>,
}

impl<K: Key, V> Agent<K, V> {
    pub fn load_blocking(&self) -> Result<&BTreeMap<K, V>, String> {
        Ok(&self.view)
    }

    pub fn insert_blocking(&mut self, key: K, value: V) -> Result<(), String> {
        self.view.insert(key, value);
        Ok(())
    }

    pub fn remove_blocking(&mut self, key: &K) -> Result<Option<V>, String> {
        Ok(self.view.remove(key))
    }
}
