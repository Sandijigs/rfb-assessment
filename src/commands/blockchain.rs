use serde::Deserialize;
use serde_json::Value;

use crate::error::AppError;
use crate::rpc::RpcClient;

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
    println!("Chain:                 {}", info.chain);
    println!("Blocks:                {}", info.blocks);
    println!("Headers:               {}", info.headers);
    println!("Difficulty:            {}", info.difficulty);
    println!("Verification progress: {:.6}", info.verification_progress);
    Ok(())
}
