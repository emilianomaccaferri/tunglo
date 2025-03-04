use std::collections::HashMap;

use clap::Parser;
use cli::TungloCli;
use config::{TungloConfig, TunnelConfig};

mod cli;
mod config;
mod tunneling;

#[tokio::main]
pub async fn main() {
    let cli = TungloCli::parse();
    let config = std::fs::read_to_string(cli.config.unwrap_or(config::DEFAULT_PATH.to_string()))
        .expect("error while reading config: ");
    let config: TungloConfig = toml::from_str(&config).unwrap();
    dbg!(&config);
}
