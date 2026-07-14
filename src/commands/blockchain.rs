use serde::Deserialize;
use serde_json::Value;

use crate::error::AppError;
use crate::rpc::RpcClient;
use crate::style::{cyan, green, magenta};

#[derive(Debug, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    pub difficulty: f64,
    #[serde(rename = "verificationprogress")]
    pub verification_progress: f64,
}

pub async fn info(client: &RpcClient) -> Result<(), AppError> {
    let info: BlockchainInfo = client.call("getblockchaininfo", Value::Array(vec![])).await?;
    println!("Chain:                 {}", magenta(&info.chain));
    println!("Blocks:                {}", green(info.blocks));
    println!("Headers:               {}", green(info.headers));
    println!("Difficulty:            {}", green(info.difficulty));
    println!(
        "Verification progress: {}",
        cyan(format!("{:.6}", info.verification_progress))
    );
    Ok(())
}
