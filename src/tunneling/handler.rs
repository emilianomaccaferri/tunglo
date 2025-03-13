use crate::{
    config::{StorageConfig, StorageType},
    storage::{self, Storage, local::LocalStorage, rqlite::RqliteStorage},
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
    pub fn new(
        to_addr: &str,
        to_port: u16,
        server_address: &str,
        server_port: u16,
        storage_config: StorageConfig,
        tx: Sender<(TunnelRunner, Channel<client::Msg>)>,
    ) -> Result<Self, TunnelError> {
        let storage = storage::get_storage(storage_config)?;
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
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // TODO: implment this!!
        info!(
            "{} got server key: {}",
            format!("{}:{}", self.server_address, self.server_port),
            server_public_key.fingerprint(Default::default())
        );
        // get server fingerprint for host
        // check if the signature matches
        // return accordingly
        Ok(true)
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
        let tunnel_runner = TunnelRunner::new(&self.to_addr, self.to_port).unwrap();
        println!("incoming connection: {_originator_address}:{_originator_port}");
        self.tx.send((tunnel_runner, channel)).await.unwrap(); // send the runner back to the        
        // Tunnel instance

        Ok(())
    }
}
