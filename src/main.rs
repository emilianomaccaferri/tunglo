mod cli;
mod tunneling;

use std::{
    collections::HashMap,
    convert::Infallible,
    future::Future,
    io::{self, Error},
    net::{AddrParseError, IpAddr, Ipv4Addr},
    sync::Arc,
};

use russh::{
    client::{self, Handler},
    keys::{load_secret_key, PrivateKeyWithHashAlg},
    Channel, ChannelMsg, ChannelStream, CryptoVec,
};
use thiserror::Error;
use tokio::{
    io::{stdin, stdout, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    select,
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex, RwLock,
    },
    task::JoinHandle,
};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let key = load_secret_key("/Users/macca/.ssh/macca-macbook", None).unwrap();
    let config = client::Config::default();
    let config = Arc::new(config);
    let mut session = client::connect(config, ("116.203.141.67", 22), ClientHandler::new(tx))
        .await
        .unwrap();
    let auth_res = session
        .authenticate_publickey(
            "macca",
            PrivateKeyWithHashAlg::new(
                Arc::new(key),
                session.best_supported_rsa_hash().await.unwrap().flatten(),
            ),
        )
        .await
        .unwrap();

    dbg!(&auth_res);
    session.tcpip_forward("0.0.0.0", 9000).await.unwrap(); // this asks the server to open the port 9000 on its side
    session.channel_open_session().await.unwrap();
    while let Some((mut tunnel, channel)) = rx.recv().await {
        tokio::spawn(async move {
            println!("new tunnel running: {}:{}", tunnel.to_addr, tunnel.to_port);
            tunnel.run(channel).await.unwrap();
        });
    }
}

struct ClientHandler {
    tx: Sender<(Tunnel, Channel<client::Msg>)>,
}
struct Tunnel {
    to_addr: IpAddr,
    to_port: u16,
    // rx: Arc<OwnedReadHalf>,
    // tx: Arc<OwnedWriteHalf>,
}
#[derive(Error, Debug)]
enum TunnelError {
    #[error("invalid address supplied: {0}")]
    InvalidAddress(String),
    #[error("io error: {1}")]
    Io(io::Error, String),
}
impl ClientHandler {
    pub fn new(tx: Sender<(Tunnel, Channel<client::Msg>)>) -> Self {
        ClientHandler { tx }
    }
}
impl From<AddrParseError> for TunnelError {
    fn from(value: AddrParseError) -> Self {
        Self::InvalidAddress(value.to_string())
    }
}
impl From<Error> for TunnelError {
    fn from(value: Error) -> Self {
        let str = value.to_string();
        Self::Io(value, str)
    }
}

impl Tunnel {
    pub fn new(to_addr: &str, to_port: u16) -> Result<Tunnel, TunnelError> {
        let to_addr = to_addr.parse()?;
        Ok(Tunnel { to_addr, to_port })
    }
    pub async fn run(
        &mut self,
        channel: Channel<client::Msg>,
    ) -> Result<JoinHandle<()>, TunnelError> {
        let mut writer = channel.make_writer();
        let mut stream = channel.into_stream();
        let conn = TcpStream::connect((self.to_addr, self.to_port)).await?;
        let (mut rx, mut tx) = conn.into_split();

        let reading_handle = tokio::spawn(async move {
            loop {
                let mut buf = vec![0u8; 4096];
                if let Ok(n) = stream.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    dbg!(&n);
                    tx.write_all(&buf[..n]).await.unwrap();
                }
            }
        });
        let writing_handle = tokio::spawn(async move {
            loop {
                let mut buf = vec![0u8; 4096];
                if let Ok(n) = rx.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    dbg!(&n);
                    writer.write_all(&buf[..n]).await.unwrap();
                }
            }
        });
        Ok(tokio::spawn(async move {
            select! {
                _w = writing_handle => {
                    println!("service disconnected");
                },
                _r = reading_handle => {
                    println!("client disconnected");
                }
            }
        }))
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
        let tunnel = Tunnel::new("127.0.0.1", 8080).unwrap();
        self.tx.send((tunnel, channel)).await.unwrap();

        Ok(())
    }
}
