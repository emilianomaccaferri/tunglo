use std::sync::Arc;

use tokio::{
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::Mutex,
};

pub(super) struct TunnelRunner<'tunnel_lifetime> {
    to_addr: &'tunnel_lifetime str,
    to_port: u16,
}
