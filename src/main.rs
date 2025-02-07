use std::{collections::HashMap, sync::Arc};

use russh::{
    client::{self, Handler},
    keys::{load_secret_key, PrivateKeyWithHashAlg},
    Channel, ChannelMsg, ChannelStream, CryptoVec,
};
use tokio::{
    io::{stdin, stdout, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    sync::{mpsc, Mutex},
};

#[tokio::main]
async fn main() {
    let key = load_secret_key("/home/macca/.ssh/macca-desktop", None).unwrap();
    let config = client::Config::default();
    let config = Arc::new(config);
    let mut session = client::connect(config, ("116.203.141.67", 22), ClientHandler {})
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
    let mut channel = session.channel_open_session().await.unwrap();
    let code = loop {
        let Some(msg) = channel.wait().await else {
            panic!("noooo");
        };
        match msg {
            ChannelMsg::Data { ref data } => {
                tokio::io::stdout().write_all(data).await.unwrap();
                tokio::io::stdout().flush().await.unwrap();
            }
            ChannelMsg::ExtendedData { ref data, ext: 1 } => {
                tokio::io::stderr().write_all(data).await.unwrap();
                tokio::io::stderr().flush().await.unwrap();
            }
            ChannelMsg::Success => (),
            ChannelMsg::Close => break 0,
            ChannelMsg::ExitStatus { exit_status } => {
                channel.eof().await.unwrap();
                break exit_status;
            }
            msg => panic!("muzunnu"),
        }
    };
}

struct ClientHandler {}

struct Tunnel {
    remote_addr: String,
    remote_port: u16,
    channel: Channel<client::Msg>,
}

impl Tunnel {
    pub async fn run(&self) {}
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
        mut channel: Channel<client::Msg>,
        connected_address: &str,
        connected_port: u32,
        originator_address: &str,
        originator_port: u32,
        session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        let sshid = String::from_utf8_lossy(session.remote_sshid());
        dbg!(
            sshid,
            connected_address,
            connected_port,
            originator_address,
            originator_port
        );
        let remote = TcpStream::connect("localhost:8080").await.unwrap();
        let (mut rx, mut tx) = remote.into_split();

        let mut writer = channel.make_writer();
        let mut stream = channel.into_stream();

        tokio::spawn(async move {
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
        tokio::spawn(async move {
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
        Ok(())
    }
}

/*struct TunnelManager {
    tunnels: HashMap<String, Tunnel>,
}
struct Tunnel {
    stream: ChannelStream<client::Msg>,
}

impl Tunnel {
    pub async fn start(&self) {
        let remote_conn = TcpStream::connect("localhost:8080").await.unwrap();
        let (mut rx, mut tx) = remote_conn.into_split();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            while let Ok(n) = rx.read(&mut buf).await {
                if n == 0 {
                    println!("stream ended");
                    break;
                }
                self.stream.write_all(&buf[..n]).await.unwrap();
            }
        });
    }
}

impl TunnelManager {
    pub async fn add_tunnel(&mut self, name: &str, channel: Channel<client::Msg>) -> &Tunnel {
        self.tunnels.insert(
            name.into(),
            Tunnel {
                stream: channel.into_stream(),
            },
        );
        self.tunnels.get(name.into()).unwrap()
    }

    pub async fn start_tunnel(&self, name: &str) {

        if let Some(tunnel) = self.tunnels.get(name) {
            let t = tunnel.clone();
            println!("starting: {}", name);
            let remote_listener = TcpStream::connect("localhost:8080").await.unwrap();
            let (mut rx, mut tx) = remote_listener.into_split();
            let writer = Arc::new(Mutex::new(t.channel.make_writer()));

            let get_reader = reader.clone();

            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                loop {
                    let mut reader = get_reader.lock().await;
                    if let Ok(n) = reader.read(&mut buf).await {
                        if n == 0 {
                            println!("stream ended");
                            break;
                        }
                        tx.write_all(&buf[..n]).await.unwrap();
                    }
                }
            });

            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                while let Ok(n) = rx.read(&mut buf).await {
                    /*if n == 0 {
                        println!("stream ended");
                        break;
                    }
                    let mut stream = tunnel_stream_send.lock().await;
                    stream.write_all(&buf[..n]).await.unwrap();*/
                }
            });
        }
    }
}
*/
