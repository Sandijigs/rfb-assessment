use serde::Deserialize;
use serde_json::Value;

use crate::error::AppError;
use crate::rpc::RpcClient;
use crate::style::{cyan, green, yellow};

#[derive(Debug, Deserialize)]
pub struct WalletInfo {
    pub walletname: String,
    pub txcount: u64,
}

#[derive(Debug, Deserialize)]
pub struct Balances {
    pub mine: MineBalance,
}

#[derive(Debug, Deserialize)]
pub struct MineBalance {
    pub trusted: f64,
    pub untrusted_pending: f64,
    #[serde(default)]
    pub immature: f64,
}

pub async fn info(client: &RpcClient) -> Result<(), AppError> {
    let info: WalletInfo = client
        .call_wallet("getwalletinfo", Value::Array(vec![]))
        .await?;
    let balances: Balances = client
        .call_wallet("getbalances", Value::Array(vec![]))
        .await?;

    let name = if info.walletname.is_empty() {
        "(default)"
    } else {
        &info.walletname
    };

    println!("Wallet name:         {}", cyan(name));
    println!("Balance:             {} BTC", green(balances.mine.trusted));
    println!(
        "Unconfirmed balance: {} BTC",
        yellow(balances.mine.untrusted_pending)
    );
    println!("Immature balance:    {} BTC", yellow(balances.mine.immature));
    println!("Transactions:        {}", green(info.txcount));
    Ok(())
}

pub async fn balance(client: &RpcClient) -> Result<(), AppError> {
    let bal: f64 = client
        .call_wallet("getbalance", Value::Array(vec![]))
        .await?;
    println!("{} BTC", green(bal));
    Ok(())
}
