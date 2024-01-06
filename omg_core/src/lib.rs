#![no_std]
extern crate alloc;

use alloc::{boxed::Box, collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use serde::{Deserialize, Serialize};
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
    fn append_blocking(&self, topic: &str, created: Option<OffsetDateTime>, data: &Value) -> Result<(), String>;
}
pub trait Key: Ord {}
impl<T: Ord> Key for T {}

pub struct Agency {
    _office: Arc<AgencyOffice>,
}

impl Agency {
    pub fn new(storage: Option<Box<dyn Storage>>) -> Agency {
        Agency {
            _office: Arc::new(AgencyOffice { _storage: storage }),
        }
    }

    pub fn agent<K: Key, V>(&self, _name: &str) -> Agent<K, V> {
        todo!("Implement: Agency::agent")
    }
}

struct AgencyOffice {
    _storage: Option<Box<dyn Storage>>,
}

pub trait State
where
    Self: Sized,
    for<'a> Self::PersistanceMsg: Deserialize<'a>,
    Self::PersistanceMsg: Serialize,
{
    type Msg;
    type PersistanceMsg;

    fn apply(self, msg: Self::Msg) -> (Self, Vec<Self::PersistanceMsg>);
    fn restore(self, msg: Self::PersistanceMsg) -> Self;
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
