use omg_core::Storage;

pub fn file(_path: &str) -> Result<Box<dyn Storage>, String> {
    Ok(Box::new(SqliteBackend{}))
}

struct SqliteBackend {

}

impl Storage for SqliteBackend {

}