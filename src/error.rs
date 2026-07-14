use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("invalid credentials — Bitcoin Core rejected the RPC auth (HTTP 401)")]
    InvalidCredentials,

    #[error("could not connect to Bitcoin Core at {url}: {source}")]
    Connection {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("wallet not found or not loaded on the node (looked for: '{0}')")]
    MissingWallet(String),

    #[error("RPC error {code}: {message}")]
    Rpc { code: i64, message: String },

    #[error("failed to parse RPC response: {0}")]
    Serde(#[from] serde_json::Error),
}
