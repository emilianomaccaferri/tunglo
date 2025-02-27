use super::{tunnel::Tunnel, tunnel_runner::TunnelRunner};
use russh::{
    client::{self, Handler},
    Channel,
};
use tokio::sync::mpsc::Sender;

pub(super) struct ClientHandler {
    tx: Sender<(TunnelRunner, Channel<client::Msg>)>,
    to_addr: String,
    to_port: u16,
}
impl ClientHandler {
    pub fn new(
        to_addr: &str,
        to_port: u16,
        tx: Sender<(TunnelRunner, Channel<client::Msg>)>,
    ) -> Self {
        ClientHandler {
            tx,
            to_addr: to_addr.to_string(),
            to_port,
        }
    }
}
impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        dbg!(&server_public_key);
        // TODO: implment this!!!
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
        self.tx.send((tunnel_runner, channel)).await.unwrap(); // send the runner back to the
                                                               // Tunnel instance

        Ok(())
    }
}
