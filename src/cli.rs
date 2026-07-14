use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "rfb-cli",
    version,
    about = "Talk to a Bitcoin Core node over JSON-RPC (Regtest via Polar)."
)]
pub struct Cli {
    /// Path to an optional TOML config file.
    #[arg(long, global = true, env = "RFB_CONFIG")]
    pub config: Option<PathBuf>,

    /// RPC endpoint URL, e.g. http://127.0.0.1:18443
    #[arg(long, global = true, env = "BITCOIN_RPC_URL")]
    pub rpc_url: Option<String>,

    /// RPC username.
    #[arg(long, global = true, env = "BITCOIN_RPC_USER")]
    pub rpc_user: Option<String>,

    /// RPC password.
    #[arg(long, global = true, env = "BITCOIN_RPC_PASSWORD")]
    pub rpc_password: Option<String>,

    /// Wallet name (required for wallet-scoped commands when multiple wallets are loaded).
    #[arg(long, global = true, env = "BITCOIN_WALLET")]
    pub wallet: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Show chain, blocks, headers, difficulty, verification progress.
    BlockchainInfo,
    /// Show wallet name, balance, unconfirmed balance, tx count.
    WalletInfo,
    /// Print the wallet balance.
    Balance,
    /// Generate and print a new receiving address.
    NewAddress,
    /// Execute an arbitrary Bitcoin Core RPC method.
    Rpc {
        /// RPC method name, e.g. getblockcount.
        method: String,
        /// Positional arguments — parsed as JSON where possible, otherwise as strings.
        params: Vec<String>,
    },
}
