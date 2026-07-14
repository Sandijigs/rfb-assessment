use serde_json::Value;

use crate::error::AppError;
use crate::rpc::RpcClient;

pub async fn new(client: &RpcClient) -> Result<(), AppError> {
    let addr: String = client
        .call_wallet("getnewaddress", Value::Array(vec![]))
        .await?;
    println!("{}", addr);
    Ok(())
}
