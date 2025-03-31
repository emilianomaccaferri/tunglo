use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rqlite_rs::prelude::{RqliteClient, RqliteClientBuilder};

use crate::tunneling::tunnel::TunnelError;

use super::{Storage, StorageError};

pub struct RqliteStorage {
    client: RqliteClient,
}
impl RqliteStorage {
    pub fn new(
        host: &str,
        user: Option<impl Into<String>>,
        password: Option<impl Into<String>>,
    ) -> Result<Self, TunnelError> {
        let client_builder = {
            if user.is_some() && password.is_some() {
                RqliteClientBuilder::new()
                    .known_host(host)
                    .auth(&user.unwrap().into(), &password.unwrap().into())
            } else {
                RqliteClientBuilder::new().known_host(host)
            }
        };

        let client = client_builder.build()?;
        Ok(RqliteStorage { client })
    }
}
#[async_trait]
impl Storage for RqliteStorage {
    async fn get_server_fingerprint(&self, address: &str) -> Result<Option<String>, StorageError> {
        Ok(Some(String::from("")))
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
