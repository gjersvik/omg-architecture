use std::sync::Arc;

pub trait Storage {
    
}

pub struct Agency {
    _storage: Option<Arc<dyn Storage>>
}

impl Agency {
    pub fn new(storage: Option<Box<dyn Storage>>) -> Agency {
        Agency {
            _storage: storage.map(Into::into)
        }
    }
}