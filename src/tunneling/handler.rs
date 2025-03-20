use crate::{
    config::StorageConfig,
    storage::{self, Storage},
};

use super::{tunnel::TunnelError, tunnel_runner::TunnelRunner};
use russh::{
    Channel,
    client::{self, Handler},
};
use tokio::sync::mpsc::Sender;
use tracing::info;

pub(super) struct ClientHandler {
    tx: Sender<(TunnelRunner, Channel<client::Msg>)>,
    to_addr: String,
    to_port: u16,
    /// these are needed for the server validation callback
    server_address: String,
    server_port: u16,
    storage: Box<dyn Storage>,
}
impl ClientHandler {
    pub async fn new(
        to_addr: &str,
        to_port: u16,
        server_address: &str,
        server_port: u16,
        storage_config: StorageConfig,
        tx: Sender<(TunnelRunner, Channel<client::Msg>)>,
    ) -> Result<Self, TunnelError> {
        let storage = storage::get_storage(storage_config)?;
        storage.ensure().await?;
        Ok(ClientHandler {
            tx,
            to_addr: to_addr.to_string(),
            to_port,
            server_address: server_address.to_string(),
            server_port,
            storage,
        })
    }
}
impl Handler for ClientHandler {
    type Error = TunnelError;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        info!(
            "{} got server key: {}",
            format!("{}:{}", self.server_address, self.server_port),
            server_public_key.fingerprint(Default::default())
        );
        // get server fingerprint for host
        // check if the signature matches
        // return accordingly
        let server_fingerprint = server_public_key
            .fingerprint(Default::default())
            .to_string();

        match self
            .storage
            .get_server_fingerprint(&self.server_address)
            .await
        {
            Ok(some_fingerprint) => {
                if let Some(stored_fingerprint) = some_fingerprint {
                    // check the stored fingerprint against the one we are getting
                    if !server_fingerprint.eq(&stored_fingerprint) {
                        tracing::error_span!("{:?} host key has changed!", self.server_address);
                        return Err(TunnelError::NastyKey);
                    }
                    tracing::info!(
                        "host key for {:?} matches the stored one",
                        self.server_address
                    );
                } else {
                    // tofu: store the key!
                    self.storage
                        .store_server_fingerprint(
                            &self.server_address,
                            &server_fingerprint.to_string(),
                        )
                        .await?;
                }
                Ok(true)
            }
            Err(e) => {
                tracing::error!("{}", e.to_string());
                Err(TunnelError::StorageLayer(e.to_string()))
            }
        }
    }
    async fn server_channel_open_forwarded_tcpip(
        &mut self,
        channel: Channel<client::Msg>,
        _connected_address: &str,
        _connected_port: u32,
        _originator_address: &str,
        _originator_port: u32,
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        let tunnel_runner = TunnelRunner::new(&self.to_addr, self.to_port)?;
        tracing::info!("incoming connection: {_originator_address}:{_originator_port}");
        self.tx.send((tunnel_runner, channel)).await.unwrap(); // send the runner back to the
        // Tunnel instance

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use russh::keys::{
        PublicKey,
        ssh_key::{Fingerprint, public::KeyData, rand_core::OsRng},
    };
    use storage::MockStorage;
    use tokio::sync::mpsc;

    use super::*;
    use mockall::predicate::*;

    fn create_public_key() -> PublicKey {
        PublicKey::from_openssh(
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAILM+rvN+ot98qgEN796jTiQfZfG1KaT0PtFDJ/XFSqti foo@bar.com",
        ).unwrap()
    }

    fn nasty_public_key() -> PublicKey {
        PublicKey::from_openssh(
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG9U2GJCV93/x/3BgfIsBGniZxit1ue9PrSU6cYmqcbo pangle@dongle.com",
        ).unwrap()
    }
    // check if the key storage/verification process works as intended
    // -> mocking the storage

    #[tokio::test]
    async fn no_fingerprint_test() {
        let (tx, _rx) = mpsc::channel(1);
        let public_key = create_public_key();
        let fingerprint = public_key.fingerprint(Default::default());
        let mut mock_storage = MockStorage::new();
        mock_storage
            .expect_get_server_fingerprint()
            .with(eq("0.0.0.0"))
            .times(1)
            .returning(|_| Ok(None)); // new host test
        mock_storage
            .expect_store_server_fingerprint()
            .with(eq("0.0.0.0"), eq(fingerprint.to_string()))
            .times(1)
            .returning(|_, __| Ok(()));

        let mut client_handler = ClientHandler {
            tx,
            to_addr: String::from("1.2.3.4"),
            to_port: 8080,
            server_address: String::from("0.0.0.0"),
            server_port: 5050,
            storage: Box::new(mock_storage),
        };

        let result = client_handler.check_server_key(&public_key).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn nasty_key_test() {
        let (tx, _rx) = mpsc::channel(1);

        let mut mock_storage = MockStorage::new();
        let nasty_key = nasty_public_key();
        mock_storage
            .expect_get_server_fingerprint()
            .with(eq("0.0.0.0"))
            .times(1)
            .returning(|_| {
                let public_key = create_public_key();
                let fingerprint = public_key.fingerprint(Default::default());
                Ok(Some(fingerprint.to_string()))
            });
        let mut client_handler = ClientHandler {
            tx,
            to_addr: String::from("1.2.3.4"),
            to_port: 8080,
            server_address: String::from("0.0.0.0"),
            server_port: 5050,
            storage: Box::new(mock_storage),
        };

        let result = client_handler.check_server_key(&nasty_key).await;
        assert!(result.is_err());
        matches!(result.err().unwrap(), TunnelError::NastyKey);
    }
    #[tokio::test]
    async fn ok_key_test() {
        let (tx, _rx) = mpsc::channel(1);

        let mut mock_storage = MockStorage::new();
        let nasty_key = create_public_key();
        mock_storage
            .expect_get_server_fingerprint()
            .with(eq("0.0.0.0"))
            .times(1)
            .returning(|_| {
                let public_key = create_public_key();
                let fingerprint = public_key.fingerprint(Default::default());
                Ok(Some(fingerprint.to_string()))
            });
        let mut client_handler = ClientHandler {
            tx,
            to_addr: String::from("1.2.3.4"),
            to_port: 8080,
            server_address: String::from("0.0.0.0"),
            server_port: 5050,
            storage: Box::new(mock_storage),
        };

        let result = client_handler.check_server_key(&nasty_key).await;
        assert!(result.is_ok());
        assert!(result.ok().unwrap());
    }
}
