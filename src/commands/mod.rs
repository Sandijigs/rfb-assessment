pub mod address;
pub mod blockchain;
pub mod wallet;

use serde_json::Value;

use crate::error::AppError;
use crate::rpc::RpcClient;

pub async fn rpc_passthrough(
    client: &RpcClient,
    method: String,
    params: Vec<String>,
) -> Result<(), AppError> {
    let json_params: Vec<Value> = params
        .into_iter()
        .map(|p| serde_json::from_str(&p).unwrap_or(Value::String(p)))
        .collect();

    let result = client.call_raw(&method, Value::Array(json_params)).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
