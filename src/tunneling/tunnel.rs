use std::net::{AddrParseError, IpAddr};

use russh::keys::PrivateKey;
use thiserror::Error;

use super::{tunnel_config::TunnelConfig, tunnel_runner::TunnelRunner};

pub(crate) struct Tunnel<'tunnel_lifetime> {
    /// tunnel name
    name: String,
    /// public address of the machine you're using for tunneling
    remote_ssh_address: String,
    /// tunneling machine ssh port
    remote_ssh_port: u16,
    /// private key for connecting to the tunneling machine
    private_key: PrivateKey,
    /// which interface the tunnel should be set on (127.0.0.1, 0.0.0.0, ...)
    remote_interface_address: String,
    /// on which port should the tunnel bind remotely (on the tunneling machine)
    remote_interface_port: u16,
    /// tunneled service's address
    to_address: String,
    /// tunneled service's port
    to_port: u16,
    /// clients connected to the tunnel
    runners: Vec<TunnelRunner<'tunnel_lifetime>>,
}
#[derive(Error, Debug)]
enum TunnelError {
    #[error("invalid address supplied: {0}")]
    InvalidAddress(String),
    #[error("io error: {1}")]
    Io(std::io::Error, String),
}
impl From<AddrParseError> for TunnelError {
    fn from(value: AddrParseError) -> Self {
        Self::InvalidAddress(value.to_string())
    }
}
impl From<std::io::Error> for TunnelError {
    fn from(value: std::io::Error) -> Self {
        let str = value.to_string();
        Self::Io(value, str)
    }
}
impl<'t> Tunnel<'t> {
    pub fn new(config: TunnelConfig) -> Result<Tunnel<'t>, TunnelError> {
        Ok(Tunnel {})
    }
    fn load_private_key(path: &str) -> Result<PrivateKey, TunnelError> {}
}
