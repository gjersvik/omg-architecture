#![no_std]
extern crate alloc;

use alloc::{collections::BTreeMap, sync::Arc, boxed::Box, string::String};

pub trait Storage: Send + Sync {}
pub trait Key: Ord {}
impl<T: Ord> Key for T {}

pub struct Agency {
    _storage: Option<Arc<dyn Storage>>,
}

impl Agency {
    pub fn new(_storage: Option<Box<dyn Storage>>) -> Agency {
        todo!("Implement: Agency::new")
    }

    pub fn get<K: Key, V>(&self, _name: &str) -> Agent<K, V> {
        todo!("Implement: Agency::get")
    }
}

pub struct Agent<K: Key, V> {
    _view: BTreeMap<K, V>,
}

impl<K: Key, V> Agent<K, V> {
    pub fn load_blocking(&self) -> Result<&BTreeMap<K, V>, String> {
        todo!("Implement: Agent::load_blocking")
    }

    pub fn insert_blocking(&mut self, _key: K, _value: V) -> Result<(), String> {
        todo!("Implement: Agent::insert_blocking")
    }

    pub fn remove_blocking(&mut self, _key: &K) -> Result<Option<V>, String> {
        todo!("Implement: Agent::insert_blocking")
    }
}
