mod cli;
mod commands;
mod config;
mod error;
mod rpc;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::cli::{Cli, Command};
use crate::config::Config;
use crate::rpc::RpcClient;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let config = Config::load(&cli)?;
    let client = RpcClient::new(config);

    match cli.command {
        Command::BlockchainInfo => commands::blockchain::info(&client).await?,
        Command::WalletInfo => commands::wallet::info(&client).await?,
        Command::Balance => commands::wallet::balance(&client).await?,
        Command::NewAddress => commands::address::new(&client).await?,
        Command::Rpc { method, params } => {
            commands::rpc_passthrough(&client, method, params).await?
        }
    }

    Ok(())
}
