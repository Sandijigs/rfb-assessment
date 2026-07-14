use reqwest::{Client, StatusCode};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::config::Config;
use crate::error::AppError;

pub struct RpcClient {
    http: Client,
    config: Config,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<Value>,
    error: Option<RpcErrorBody>,
    #[allow(dead_code)]
    id: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct RpcErrorBody {
    code: i64,
    message: String,
}

impl RpcClient {
    pub fn new(config: Config) -> Self {
        Self {
            http: Client::new(),
            config,
        }
    }

    fn base_url(&self) -> String {
        self.config.rpc_url.clone()
    }

    fn wallet_url(&self) -> String {
        match &self.config.wallet {
            Some(w) => format!("{}/wallet/{}", self.config.rpc_url.trim_end_matches('/'), w),
            None => self.base_url(),
        }
    }

    async fn post(&self, url: &str, method: &str, params: Value) -> Result<Value, AppError> {
        let body = json!({
            "jsonrpc": "1.0",
            "id": "rfb-cli",
            "method": method,
            "params": params,
        });

        tracing::debug!(url, method, ?params, "rpc call");

        let resp = self
            .http
            .post(url)
            .basic_auth(&self.config.rpc_user, Some(&self.config.rpc_password))
            .json(&body)
            .send()
            .await
            .map_err(|source| {
                if source.is_connect() || source.is_timeout() {
                    AppError::Connection { url: url.to_string(), source }
                } else {
                    AppError::Http(source)
                }
            })?;

        if resp.status() == StatusCode::UNAUTHORIZED {
            return Err(AppError::InvalidCredentials);
        }

        let status = resp.status();
        let text = resp.text().await?;

        let parsed: RpcResponse = serde_json::from_str(&text).map_err(|e| {
            AppError::Config(format!(
                "unexpected response from node (status {status}): {text} :: {e}"
            ))
        })?;

        if let Some(err) = parsed.error {
            // -18: wallet not found, -19: wallet not loaded
            if err.code == -18 || err.code == -19 {
                return Err(AppError::MissingWallet(
                    self.config.wallet.clone().unwrap_or_default(),
                ));
            }
            return Err(AppError::Rpc { code: err.code, message: err.message });
        }

        parsed
            .result
            .ok_or_else(|| AppError::Config("RPC returned no result and no error".into()))
    }

    /// Call a wallet-agnostic RPC and decode the result into T.
    pub async fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, AppError> {
        let value = self.post(&self.base_url(), method, params).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Call a wallet-scoped RPC (uses `/wallet/<name>` if a wallet is configured).
    pub async fn call_wallet<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, AppError> {
        let value = self.post(&self.wallet_url(), method, params).await?;
        Ok(serde_json::from_value(value)?)
    }

    /// Passthrough for the generic `rpc` subcommand — returns raw JSON.
    pub async fn call_raw(&self, method: &str, params: Value) -> Result<Value, AppError> {
        self.post(&self.wallet_url(), method, params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn client(url: &str, wallet: Option<&str>) -> RpcClient {
        RpcClient::new(Config {
            rpc_url: url.into(),
            rpc_user: "u".into(),
            rpc_password: "p".into(),
            wallet: wallet.map(String::from),
        })
    }

    #[test]
    fn wallet_url_uses_base_when_no_wallet_is_set() {
        let c = client("http://127.0.0.1:18443", None);
        assert_eq!(c.wallet_url(), "http://127.0.0.1:18443");
    }

    #[test]
    fn wallet_url_appends_wallet_name() {
        let c = client("http://127.0.0.1:18443", Some("myw"));
        assert_eq!(c.wallet_url(), "http://127.0.0.1:18443/wallet/myw");
    }

    #[test]
    fn wallet_url_strips_trailing_slash_from_base() {
        let c = client("http://127.0.0.1:18443/", Some("myw"));
        assert_eq!(c.wallet_url(), "http://127.0.0.1:18443/wallet/myw");
    }
}
