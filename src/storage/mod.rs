use async_trait::async_trait;
use local::LocalStorage;
use rqlite::RqliteStorage;
use thiserror::Error;

#[cfg(test)]
use mockall::automock;

use crate::{
    config::{StorageConfig, StorageType},
    tunneling::tunnel::TunnelError,
};
pub(crate) mod local;
pub(crate) mod rqlite;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("sqlite returned an error: {1}")]
    LocalSqlite(rusqlite::Error, String),
}

#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait Storage: Send + Sync {
    async fn get_server_fingerprint(&self, address: &str) -> Result<Option<String>, StorageError>;
    async fn store_server_fingerprint(
        &self,
        address: &str,
        fingerprint: &str,
    ) -> Result<(), StorageError>;
    async fn ensure(&self) -> Result<(), StorageError>;
}

pub fn get_storage(storage_config: StorageConfig) -> Result<Box<dyn Storage>, TunnelError> {
    match storage_config.storage_type {
        StorageType::Rqlite => {
            if let Some(config) = storage_config.rqlite {
                Ok(Box::new(RqliteStorage::new(
                    config.host.get(),
                    config.user,
                    config.password,
                )?))
            } else {
                Err(TunnelError::NoRqliteConfig)
            }
        }
        StorageType::Local => Ok(Box::new(LocalStorage::new()?)),
    }
}

impl From<StorageError> for TunnelError {
    fn from(err: StorageError) -> Self {
        TunnelError::StorageLayer(err.to_string())
    }
}
