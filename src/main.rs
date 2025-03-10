use clap::Parser;
use cli::TungloCli;
use config::TungloConfig;
use futures::future::join_all;
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
    let mut tunnels: Vec<Tunnel> = loaded_config
        .tunnels
        .into_iter()
        .map(|c| Tunnel::new(c).unwrap())
        .collect();

    let mut handlers = vec![];

    for i in 0..tunnels.len() {
        if let Some(tunnel) = tunnels.get_mut(i) {
            handlers.push(tunnel.connect().await.unwrap());
        }
    }
    join_all(handlers).await;
    Ok(())
}
