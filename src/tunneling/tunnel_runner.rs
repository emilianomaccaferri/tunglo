use russh::{Channel, client};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    task::JoinHandle,
};

use super::tunnel::TunnelError;

pub(super) struct TunnelRunner {
    to_addr: String,
    to_port: u16,
}

impl TunnelRunner {
    pub fn addr(&self) -> &str {
        &self.to_addr
    }
    pub fn port(&self) -> u16 {
        self.to_port
    }
    pub fn new(to_addr: &str, to_port: u16) -> Result<TunnelRunner, TunnelError> {
        Ok(TunnelRunner {
            to_addr: to_addr.to_string(),
            to_port,
        })
    }
    pub async fn run(
        &mut self,
        channel: Channel<client::Msg>,
    ) -> Result<JoinHandle<()>, TunnelError> {
        let mut writer = channel.make_writer();
        let mut stream = channel.into_stream();
        let conn = TcpStream::connect((self.to_addr.to_string(), self.to_port)).await?;
        let (mut rx, mut tx) = conn.into_split();

        let reading_handle = tokio::spawn(async move {
            loop {
                let mut buf = vec![0u8; 4096];
                if let Ok(n) = stream.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    // dbg!(&n);
                    if let Err(e) = tx.write_all(&buf[..n]).await {
                        eprintln!("bad write: {:?}", e);
                        break;
                    }
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
                    if let Err(e) = writer.write_all(&buf[..n]).await {
                        eprintln!("bad write: {:?}", e);
                        break;
                    };
                }
            }
        });
        Ok(tokio::spawn(async move {
            tokio::select! {
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
