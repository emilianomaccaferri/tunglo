use std::{env::VarError, net::AddrParseError, sync::Arc};

use russh::{
    client,
    keys::{PrivateKey, PrivateKeyWithHashAlg, load_secret_key},
};
use thiserror::Error;

use crate::{
    config::{PrivateKeyPassphrase, TunnelConfig},
    tunneling::handler::ClientHandler,
};

use super::tunnel_runner::TunnelRunner;

pub(crate) struct Tunnel {
    /// tunnel name
    name: String,
    /// public address of the machine you're using for tunneling
    remote_ssh_address: String,
    /// tunneling machine ssh port
    remote_ssh_port: u16,
    /// the ssh user
    remote_ssh_user: String,
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
    runners: Vec<TunnelRunner>,
}
#[derive(Error, Debug)]
pub enum TunnelError {
    #[error("invalid address supplied: {0}")]
    InvalidAddress(String),
    #[error("io error: {1}")]
    Io(std::io::Error, String),
    #[error("private key error: {1}")]
    PrivateKey(russh::keys::Error, String),
    #[error("env variable for private key error: {0}")]
    EnvError(String),
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
impl From<russh::keys::Error> for TunnelError {
    fn from(value: russh::keys::Error) -> Self {
        let str_val = value.to_string();
        Self::PrivateKey(value, str_val)
    }
}

impl Tunnel {
    pub fn new(config: TunnelConfig) -> Result<Tunnel, TunnelError> {
        let private_key =
            Tunnel::load_private_key(&config.private_key_path, &config.private_key_passphrase)?;
        Ok(Tunnel {
            name: config.name,
            private_key,
            remote_interface_address: config.remote_interface_address,
            remote_interface_port: config.remote_interface_port,
            remote_ssh_address: config.remote_ssh_address,
            remote_ssh_port: config.remote_ssh_port,
            remote_ssh_user: config.remote_ssh_user,
            to_address: config.to_address,
            to_port: config.to_port,
            runners: Vec::new(),
        })
    }
    pub async fn connect(&self) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let config = client::Config::default();
        let config = Arc::new(config);
        let mut session = client::connect(
            config,
            (self.remote_ssh_address.to_owned(), self.remote_ssh_port),
            ClientHandler::new(&self.to_address, self.to_port, tx),
        )
        .await
        .unwrap();
        let auth_res = session
            .authenticate_publickey(
                self.remote_ssh_user.clone(),
                PrivateKeyWithHashAlg::new(
                    Arc::new(self.private_key.to_owned()),
                    session.best_supported_rsa_hash().await.unwrap().flatten(),
                ),
            )
            .await
            .unwrap();

        dbg!(&auth_res);
        session
            .tcpip_forward(
                self.remote_interface_address.to_owned(),
                self.remote_interface_port as u32, // u32 for some reason??
            )
            .await
            .unwrap(); // this asks the server to open the specified port on the remote interface
        session.channel_open_session().await.unwrap();
        while let Some((mut runner, channel)) = rx.recv().await {
            tokio::spawn(async move {
                println!("new tunnel running: {}:{}", runner.addr(), runner.port());
                runner.run(channel).await.unwrap();
            });
        }
    }
    fn load_private_key(
        key_path: &str,
        passphrase: &Option<PrivateKeyPassphrase>,
    ) -> Result<PrivateKey, TunnelError> {
        if let Some(passphrase) = passphrase {
            match passphrase {
                PrivateKeyPassphrase {
                    value: Some(plaintext_key),
                    from_env: None,
                } => Ok(load_secret_key(key_path, Some(plaintext_key))?),
                PrivateKeyPassphrase {
                    from_env: Some(env_var),
                    value: None,
                } => {
                    let env_value = std::env::var(env_var).map_err(|e| match e {
                        VarError::NotPresent => TunnelError::EnvError(
                            "{env_var} not found in the environment!".to_string(),
                        ),
                        VarError::NotUnicode(_) => {
                            TunnelError::EnvError("{env_var} is not unicode!".to_string())
                        }
                    })?;
                    Ok(load_secret_key(key_path, Some(&env_value))?)
                }
                _ => Ok(load_secret_key(key_path, None)?),
            }
        } else {
            Ok(load_secret_key(key_path, None)?)
        }
    }
}
