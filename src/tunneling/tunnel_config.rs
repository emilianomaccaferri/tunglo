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
    pub tun_type: TunnelType,
}
pub(crate) enum TunnelType {
    Http,
    Http2,
    Generic,
}
pub(crate) enum PrivateKeyPassphrase {
    /// the private key is stored in platintext inside the tunnel configuration file
    /// (passhphrase-value)
    PlainText(String),
    /// the private key must be fetched from an environmental variable (env-var-name)
    Environment(String),
}
