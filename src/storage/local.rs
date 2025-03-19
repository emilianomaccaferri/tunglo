use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rusqlite::Connection;

use super::{Storage, StorageError};
pub struct LocalStorage {
    connection: Arc<Mutex<rusqlite::Connection>>,
}
impl LocalStorage {
    pub fn new() -> Result<Self, StorageError> {
        Ok(LocalStorage {
            connection: Arc::new(Mutex::new(Connection::open("./data/known_hosts.db")?)),
        })
    }
}
#[async_trait]
impl Storage for LocalStorage {
    async fn get_server_fingerprint(&self, address: &str) -> Result<Option<String>, StorageError> {
        let conn = self.connection.clone();
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare("select fingerprint from known_hosts where hostname = ?1")?;
        let mut query_mapped = stmt.query_map([address], |row| row.get(0))?;
        if let Some(v) = query_mapped.next() {
            Ok(Some(v?))
        } else {
            Ok(None)
        }
    }
    async fn store_server_fingerprint(
        &self,
        address: &str,
        fingerprint: &str,
    ) -> Result<(), StorageError> {
        let conn = self.connection.clone();
        let conn = conn.lock().unwrap();
        tracing::info!("storing fingerprint for {:?}", address);
        conn.execute(
            "insert into known_hosts values (?1, ?2)",
            (address, fingerprint),
        )?;
        Ok(())
    }
    async fn ensure(&self) -> Result<(), StorageError> {
        let conn = self.connection.clone();
        let conn = conn.lock().unwrap();
        conn.execute(r#"
            create table if not exists known_hosts(hostname varchar(255) primary key, fingerprint varchar(255) not null);
        "#, ())?;
        Ok(())
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(value: rusqlite::Error) -> Self {
        let str_value = value.to_string();
        StorageError::LocalSqlite(value, str_value)
    }
}
