use async_trait::async_trait;

use super::{Storage, StorageError};

pub struct RqliteStorage;
impl RqliteStorage {
    pub fn new(host: &str, user: Option<String>, password: Option<String>) -> Self {
        RqliteStorage {}
    }
}
#[async_trait]
impl Storage for RqliteStorage {
    async fn get_server_fingerprint(&self, address: &str) -> Result<Option<String>, StorageError> {
        Ok(Some(String::from("ciao")))
    }
    async fn store_server_fingerprint(
        &self,
        address: &str,
        fingerprint: &str,
    ) -> Result<(), StorageError> {
        Ok(())
    }
    async fn ensure(&self) -> Result<(), StorageError> {
        Ok(())
    }
}
