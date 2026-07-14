use serde::Deserialize;
use std::fs;

use crate::cli::Cli;
use crate::error::AppError;

const DEFAULT_URL: &str = "http://127.0.0.1:18443";
const DEFAULT_USER: &str = "polaruser";
const DEFAULT_PASSWORD: &str = "polarpass";

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub rpc_user: String,
    pub rpc_password: String,
    pub wallet: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    rpc_url: Option<String>,
    rpc_user: Option<String>,
    rpc_password: Option<String>,
    wallet: Option<String>,
}

impl Config {
    pub fn load(cli: &Cli) -> Result<Self, AppError> {
        let file = match &cli.config {
            Some(path) => {
                let text = fs::read_to_string(path)?;
                toml::from_str::<FileConfig>(&text)
                    .map_err(|e| AppError::Config(format!("invalid TOML config: {e}")))?
            }
            None => FileConfig::default(),
        };
        Ok(Self::from_parts(cli, file))
    }

    fn from_parts(cli: &Cli, file: FileConfig) -> Self {
        Config {
            rpc_url: cli
                .rpc_url
                .clone()
                .or(file.rpc_url)
                .unwrap_or_else(|| DEFAULT_URL.to_string()),
            rpc_user: cli
                .rpc_user
                .clone()
                .or(file.rpc_user)
                .unwrap_or_else(|| DEFAULT_USER.to_string()),
            rpc_password: cli
                .rpc_password
                .clone()
                .or(file.rpc_password)
                .unwrap_or_else(|| DEFAULT_PASSWORD.to_string()),
            wallet: cli.wallet.clone().or(file.wallet),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Command;

    fn empty_cli() -> Cli {
        Cli {
            config: None,
            rpc_url: None,
            rpc_user: None,
            rpc_password: None,
            wallet: None,
            command: Command::BlockchainInfo,
        }
    }

    #[test]
    fn defaults_apply_when_nothing_is_set() {
        let config = Config::from_parts(&empty_cli(), FileConfig::default());
        assert_eq!(config.rpc_url, DEFAULT_URL);
        assert_eq!(config.rpc_user, DEFAULT_USER);
        assert_eq!(config.rpc_password, DEFAULT_PASSWORD);
        assert_eq!(config.wallet, None);
    }

    #[test]
    fn file_overrides_defaults() {
        let file = FileConfig {
            rpc_url: Some("http://from-file:1234".into()),
            rpc_user: Some("fileuser".into()),
            rpc_password: Some("filepass".into()),
            wallet: Some("filewallet".into()),
        };
        let config = Config::from_parts(&empty_cli(), file);
        assert_eq!(config.rpc_url, "http://from-file:1234");
        assert_eq!(config.rpc_user, "fileuser");
        assert_eq!(config.rpc_password, "filepass");
        assert_eq!(config.wallet, Some("filewallet".into()));
    }

    #[test]
    fn cli_flags_override_file_where_set() {
        let cli = Cli {
            config: None,
            rpc_url: Some("http://from-cli:9999".into()),
            rpc_user: Some("cliuser".into()),
            rpc_password: None, // fall through to file
            wallet: None,       // fall through to file
            command: Command::BlockchainInfo,
        };
        let file = FileConfig {
            rpc_url: Some("http://from-file:1234".into()),
            rpc_user: Some("fileuser".into()),
            rpc_password: Some("filepass".into()),
            wallet: Some("filewallet".into()),
        };
        let config = Config::from_parts(&cli, file);
        assert_eq!(config.rpc_url, "http://from-cli:9999");
        assert_eq!(config.rpc_user, "cliuser");
        assert_eq!(config.rpc_password, "filepass");
        assert_eq!(config.wallet, Some("filewallet".into()));
    }

    #[test]
    fn toml_parses_expected_keys() {
        let toml_text = r#"
            rpc_url = "http://example:1234"
            rpc_user = "alice"
            rpc_password = "hunter2"
            wallet = "my-wallet"
        "#;
        let parsed: FileConfig = toml::from_str(toml_text).expect("valid TOML");
        assert_eq!(parsed.rpc_url.as_deref(), Some("http://example:1234"));
        assert_eq!(parsed.rpc_user.as_deref(), Some("alice"));
        assert_eq!(parsed.rpc_password.as_deref(), Some("hunter2"));
        assert_eq!(parsed.wallet.as_deref(), Some("my-wallet"));
    }
}
