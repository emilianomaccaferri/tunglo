use std::collections::HashMap;

use serde::Deserialize;

pub const DEFAULT_PATH: &str = "~/.config/tunglo.toml";
#[derive(Deserialize, Debug)]
pub(crate) struct TungloConfig {
    pub tunnels: Vec<TunnelConfig>,
}
#[derive(Deserialize, Debug)]
pub(crate) struct TunnelConfig {
    pub name: String,
    pub remote_ssh_address: String,
    pub remote_ssh_port: u16,
    pub remote_ssh_user: String,
    pub private_key_path: String,
    pub private_key_passphrase: Option<PrivateKeyPassphrase>,
    pub remote_interface_address: String,
    pub remote_interface_port: u16,
    pub to_address: String,
    pub to_port: u16,
    #[serde(rename = "type")]
    pub tun_type: TunnelType,
}
#[derive(Deserialize, Debug)]
pub(crate) enum TunnelType {
    #[serde(alias = "http", alias = "HTTP")]
    Http,
    #[serde(alias = "http2", alias = "HTTP2")]
    Http2,
    #[serde(alias = "generic", alias = "GENERIC")]
    Generic,
}
#[derive(Deserialize, Debug)]
pub(crate) struct PrivateKeyPassphrase {
    /// the private key is stored in platintext inside the tunnel configuration file
    /// (passhphrase-value)
    pub value: Option<String>,
    /// the private key must be fetched from an environmental variable (env-var-name)
    pub from_env: Option<String>,
}
