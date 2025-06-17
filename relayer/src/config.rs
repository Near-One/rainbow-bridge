use color_eyre::{Result, eyre::Context};
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use near_primitives::types::AccountId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::constants::defaults;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Ethereum beacon node configuration
    #[serde(default)]
    pub beacon: BeaconConfig,

    /// Ethereum execution layer configuration
    #[serde(default)]
    pub execution: ExecutionConfig,

    /// NEAR blockchain configuration
    #[serde(default)]
    pub near: NearConfig,

    /// Relayer operation configuration
    #[serde(default)]
    pub relayer: RelayerConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconConfig {
    /// Beacon node HTTP API endpoint
    #[serde(default)]
    pub endpoint: String,

    /// Timeout for beacon API requests in seconds
    #[serde(default)]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Ethereum execution RPC endpoint
    #[serde(default)]
    pub endpoint: String,

    /// Timeout for execution RPC requests in seconds
    #[serde(default)]
    pub timeout_secs: u64,

    /// Maximum number of blocks to fetch in one batch
    #[serde(default)]
    pub max_batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearConfig {
    /// NEAR RPC endpoint
    #[serde(default)]
    pub endpoint: String,

    /// NEAR contract account ID
    pub contract_account_id: String,

    /// NEAR signer account ID
    pub signer_account_id: String,

    /// NEAR signer secret key
    pub secret_key: String,

    /// Timeout for NEAR RPC requests in seconds
    #[serde(default)]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerConfig {
    /// Number of epochs between light client update submissions
    #[serde(default)]
    pub update_interval_epochs: u64,

    /// Maximum number of headers to submit in one batch
    #[serde(default)]
    pub headers_batch_size: usize,

    /// Maximum number of headers to process in one period (to limit the 8192 headers case)
    #[serde(default)]
    pub max_headers_per_period: usize,

    /// Sleep duration when synced (seconds)
    #[serde(default)]
    pub sync_sleep_secs: u64,

    /// Sleep duration after submission (seconds)
    #[serde(default)]
    pub submission_sleep_secs: u64,

    /// Maximum number of iterations (for testing, None = infinite)
    pub max_iterations: Option<u64>,

    /// Enable dry run mode (don't actually submit transactions)
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error
    #[serde(default)]
    pub level: String,

    /// Enable JSON formatted logs
    #[serde(default)]
    pub json: bool,

    /// Log file path (None = stdout only)
    pub file: Option<PathBuf>,
}

impl Default for BeaconConfig {
    fn default() -> Self {
        Self {
            endpoint: defaults::BEACON_ENDPOINT.to_string(),
            timeout_secs: defaults::TIMEOUT_SECS,
        }
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            endpoint: defaults::EXECUTION_ENDPOINT.to_string(),
            timeout_secs: defaults::TIMEOUT_SECS,
            max_batch_size: defaults::EXECUTION_BATCH_SIZE,
        }
    }
}

impl Default for NearConfig {
    fn default() -> Self {
        Self {
            endpoint: defaults::NEAR_ENDPOINT.to_string(),
            contract_account_id: defaults::CONTRACT_ACCOUNT_ID.to_string(),
            signer_account_id: defaults::SIGNER_ACCOUNT_ID.to_string(),
            secret_key: String::new(),
            timeout_secs: defaults::TIMEOUT_SECS,
        }
    }
}

impl Default for RelayerConfig {
    fn default() -> Self {
        Self {
            update_interval_epochs: defaults::UPDATE_INTERVAL_EPOCHS,
            headers_batch_size: defaults::HEADERS_BATCH_SIZE,
            max_headers_per_period: defaults::MAX_HEADERS_PER_PERIOD,
            sync_sleep_secs: defaults::SYNC_SLEEP_SECS,
            submission_sleep_secs: defaults::SUBMISSION_SLEEP_SECS,
            max_iterations: None,
            dry_run: false,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: defaults::LOG_LEVEL.to_string(),
            json: false,
            file: None,
        }
    }
}

impl Config {
    /// Load configuration from multiple sources using Figment
    ///
    /// Priority (highest to lowest):
    /// 1. Environment variables (prefixed with RELAYER_)
    /// 2. Config file (if provided, or relayer.toml if it exists)
    /// 3. Default values
    pub fn load(config_file: Option<PathBuf>) -> Result<Self> {
        let mut figment = Figment::from(Serialized::defaults(Config::default()));

        // Add config file if provided, or check for default relayer.toml
        let config_path = config_file.or_else(|| {
            let default_path = PathBuf::from("relayer.toml");
            if default_path.exists() {
                Some(default_path)
            } else {
                None
            }
        });

        if let Some(path) = config_path {
            figment = figment.merge(Toml::file(&path));
        }

        // Add environment variables with RELAYER_ prefix
        figment = figment.merge(Env::prefixed("RELAYER_").split("__"));

        figment
            .extract()
            .wrap_err("Failed to extract configuration from sources")
    }

    /// Parse NEAR account IDs with validation
    pub fn parse_near_accounts(&self) -> Result<(AccountId, AccountId)> {
        let contract_account_id: AccountId =
            self.near.contract_account_id.parse().with_context(|| {
                format!(
                    "Invalid contract account ID '{}'",
                    self.near.contract_account_id
                )
            })?;

        let signer_account_id: AccountId =
            self.near.signer_account_id.parse().with_context(|| {
                format!(
                    "Invalid signer account ID '{}'",
                    self.near.signer_account_id
                )
            })?;

        Ok((contract_account_id, signer_account_id))
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate NEAR account IDs
        self.parse_near_accounts()
            .wrap_err("Failed to validate NEAR account IDs")?;

        // Validate secret key is provided
        if self.near.secret_key.is_empty() {
            return Err(color_eyre::eyre::eyre!("secret_key must be provided"));
        }

        // Validate log level
        match self.logging.level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(color_eyre::eyre::eyre!(
                    "Invalid log level: {}",
                    self.logging.level
                ));
            }
        }

        // Validate numeric values
        if self.relayer.update_interval_epochs == 0 {
            return Err(color_eyre::eyre::eyre!(
                "update_interval_epochs must be greater than 0"
            ));
        }

        if self.relayer.headers_batch_size == 0 {
            return Err(color_eyre::eyre::eyre!(
                "headers_batch_size must be greater than 0"
            ));
        }

        if self.relayer.max_headers_per_period == 0 {
            return Err(color_eyre::eyre::eyre!(
                "max_headers_per_period must be greater than 0"
            ));
        }

        Ok(())
    }

    /// Print configuration summary (hiding sensitive information)
    pub fn print_summary(&self) {
        tracing::info!("🎯 Configuration Summary:");
        tracing::info!("  Beacon endpoint: {}", self.beacon.endpoint);
        tracing::info!("  Execution endpoint: {}", self.execution.endpoint);
        tracing::info!("  NEAR endpoint: {}", self.near.endpoint);
        tracing::info!("  Contract account: {}", self.near.contract_account_id);
        tracing::info!("  Signer account: {}", self.near.signer_account_id);
        tracing::info!("  Secret key: <provided>");
        tracing::info!(
            "  Update interval: {} epochs",
            self.relayer.update_interval_epochs
        );
        tracing::info!("  Batch size: {} headers", self.relayer.headers_batch_size);
        tracing::info!("  Max headers per period: {}", self.relayer.max_headers_per_period);
        tracing::info!("  Sync sleep: {}s", self.relayer.sync_sleep_secs);
        tracing::info!(
            "  Submission sleep: {}s",
            self.relayer.submission_sleep_secs
        );
        tracing::info!("  Log level: {}", self.logging.level);
        tracing::info!("  Dry run: {}", self.relayer.dry_run);

        if let Some(max_iter) = self.relayer.max_iterations {
            tracing::info!("  Max iterations: {}", max_iter);
        }
    }

    /// Generate example config file content
    pub fn example_toml() -> Result<String> {
        toml::to_string_pretty(&Config::default())
            .wrap_err("Failed to serialize default configuration to TOML")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.beacon.endpoint, defaults::BEACON_ENDPOINT);
        assert_eq!(
            config.relayer.headers_batch_size,
            defaults::HEADERS_BATCH_SIZE
        );
        assert_eq!(config.logging.level, defaults::LOG_LEVEL);
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        // This will fail because default account IDs are not valid and secret key file doesn't exist
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_example_toml_generation() {
        let toml = Config::example_toml().unwrap();
        assert!(toml.contains("[beacon]"));
        assert!(toml.contains("[near]"));
        assert!(toml.contains("[relayer]"));
        assert!(toml.contains("[logging]"));
    }
}
