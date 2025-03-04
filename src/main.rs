use std::collections::HashMap;

use clap::Parser;
use cli::TungloCli;
use config::{TungloConfig, TunnelConfig};
use futures::{StreamExt, future::join_all, stream::FuturesUnordered};
use tunneling::tunnel::{Tunnel, TunnelError};

mod cli;
mod config;
mod tunneling;

#[tokio::main]
pub async fn main() -> Result<(), TunnelError> {
    let cli = TungloCli::parse();
    let config = std::fs::read_to_string(cli.config.unwrap_or(config::DEFAULT_PATH.to_string()))
        .expect("error while reading config: ");
    let loaded_config: TungloConfig = toml::from_str(&config).unwrap();
    let tunnels: Vec<Tunnel> = loaded_config
        .tunnels
        .into_iter()
        .map(|c| Tunnel::new(c).unwrap())
        .collect();

    // let mut handlers = vec![];

    for tun in tunnels {
        println!("{}", tun.name())
        // let mut rx = tun.connect().await?;
        // println!("tunnel {}: connected", tun.name());
        // // let handler = tokio::spawn(async move {
        // // wait for connections on another tokio task
        // while let Some((mut runner, channel)) = rx.recv().await {
        //     println!("waiting...");
        //     tokio::spawn(async move {
        //         println!("new tunnel running: {}:{}", runner.addr(), runner.port());
        //         runner.run(channel).await.expect("runner error: ");
        //     });
        // }
        // });
        // handler.await;
        // handlers.push(handler);
    }
    // join_all(handlers).await;

    Ok(())
}
