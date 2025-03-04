use russh::{Channel, client};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Join},
    net::TcpStream,
    task::JoinHandle,
};

use super::tunnel::{Tunnel, TunnelError};

pub(crate) struct TunnelRunner {
    to_addr: String,
    to_port: u16,
}
type RunResult = tokio::task::JoinHandle<
    std::result::Result<std::result::Result<(), TunnelError>, tokio::task::JoinError>,
>;
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
    pub async fn run(&mut self, channel: Channel<client::Msg>) -> Result<RunResult, TunnelError> {
        let mut writer = channel.make_writer();
        let mut stream = channel.into_stream();
        let conn = TcpStream::connect((self.to_addr.to_string(), self.to_port)).await?;
        let (mut rx, mut tx) = conn.into_split();

        let reading_handle: JoinHandle<Result<(), TunnelError>> = tokio::spawn(async move {
            loop {
                let mut buf = vec![0u8; 4096];
                if let Ok(n) = stream.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    // dbg!(&n);
                    if let Err(e) = tx.write_all(&buf[..n]).await {
                        return Err(TunnelError::Io(e, "bad_read".to_string()));
                    }
                }
            }
            Ok(())
        });
        let writing_handle: JoinHandle<Result<(), TunnelError>> = tokio::spawn(async move {
            loop {
                let mut buf = vec![0u8; 4096];
                if let Ok(n) = rx.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    dbg!(&n);
                    if let Err(e) = writer.write_all(&buf[..n]).await {
                        return Err(TunnelError::Io(e, "bad_write".to_string()));
                    };
                }
            }
            Ok(())
        });
        let select_future = tokio::spawn(async move {
            tokio::select! {
                write_result = writing_handle => {
                    write_result
                },
                read_result = reading_handle => {
                    read_result
                }
            }
        });
        Ok(select_future)
    }
}
