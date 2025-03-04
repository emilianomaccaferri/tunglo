use clap::Parser;

#[derive(Parser)]
#[command(version, about = "Tunglo creates reliable SSH tunnels that you can use for your cloudnative services", long_about = None)]
pub(crate) struct TungloCli {
    /// custom config file
    #[arg(short, long)]
    pub config: Option<String>,
}
