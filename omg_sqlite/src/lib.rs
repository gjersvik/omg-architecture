use omg_core::Storage;

pub fn file(_path: &str) -> Box<dyn Storage> {
    Box::new(SqliteBackend{})
}

struct SqliteBackend {

}

impl Storage for SqliteBackend {

}