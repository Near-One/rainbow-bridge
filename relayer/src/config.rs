use color_eyre::{Result, eyre::Context};
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use near_primitives::types::AccountId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Ethereum beacon node configuration
    pub beacon: BeaconConfig,

    /// Ethereum execution layer configuration
    pub execution: ExecutionConfig,

    /// NEAR blockchain configuration
    pub near: NearConfig,

    /// Relayer operation configuration
    pub relayer: RelayerConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconConfig {
    /// Beacon node HTTP API endpoint
    #[serde(default = "default_beacon_endpoint")]
    pub endpoint: String,

    /// Timeout for beacon API requests in seconds
    #[serde(default = "default_beacon_timeout")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Ethereum execution RPC endpoint
    #[serde(default = "default_execution_endpoint")]
    pub endpoint: String,

    /// Timeout for execution RPC requests in seconds
    #[serde(default = "default_execution_timeout")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearConfig {
    /// NEAR RPC endpoint
    #[serde(default = "default_near_endpoint")]
    pub endpoint: String,

    /// NEAR contract account ID
    pub contract_account_id: String,

    /// NEAR signer account ID
    pub signer_account_id: String,

    /// Path to NEAR signer secret key file
    pub secret_key_path: PathBuf,

    /// Timeout for NEAR RPC requests in seconds
    #[serde(default = "default_near_timeout")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerConfig {
    /// Number of epochs between light client update submissions
    #[serde(default = "default_update_interval_epochs")]
    pub update_interval_epochs: u64,

    /// Maximum number of headers to submit in one batch
    #[serde(default = "default_headers_batch_size")]
    pub headers_batch_size: usize,

    /// Sleep duration when synced (seconds)
    #[serde(default = "default_sync_sleep_secs")]
    pub sync_sleep_secs: u64,

    /// Sleep duration after submission (seconds)
    #[serde(default = "default_submission_sleep_secs")]
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
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Enable JSON formatted logs
    #[serde(default)]
    pub json: bool,

    /// Log file path (None = stdout only)
    pub file: Option<PathBuf>,
}

// Default value functions
fn default_beacon_endpoint() -> String {
    "http://unstable.sepolia.beacon-api.nimbus.team".to_string()
}

fn default_execution_endpoint() -> String {
    "https://ethereum-sepolia-rpc.publicnode.com".to_string()
}

fn default_near_endpoint() -> String {
    "https://rpc.testnet.near.org".to_string()
}

fn default_beacon_timeout() -> u64 {
    30
}

fn default_execution_timeout() -> u64 {
    30
}

fn default_near_timeout() -> u64 {
    30
}

fn default_update_interval_epochs() -> u64 {
    1
}

fn default_headers_batch_size() -> usize {
    32
}

fn default_sync_sleep_secs() -> u64 {
    60
}

fn default_submission_sleep_secs() -> u64 {
    12
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            beacon: BeaconConfig::default(),
            execution: ExecutionConfig::default(),
            near: NearConfig::default(),
            relayer: RelayerConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for BeaconConfig {
    fn default() -> Self {
        Self {
            endpoint: default_beacon_endpoint(),
            timeout_secs: default_beacon_timeout(),
        }
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            endpoint: default_execution_endpoint(),
            timeout_secs: default_execution_timeout(),
        }
    }
}

impl Default for NearConfig {
    fn default() -> Self {
        Self {
            endpoint: default_near_endpoint(),
            contract_account_id: "eth-client.testnet".to_string(),
            signer_account_id: "relayer.testnet".to_string(),
            secret_key_path: PathBuf::from("./keys/signer.txt"),
            timeout_secs: default_near_timeout(),
        }
    }
}

impl Default for RelayerConfig {
    fn default() -> Self {
        Self {
            update_interval_epochs: default_update_interval_epochs(),
            headers_batch_size: default_headers_batch_size(),
            sync_sleep_secs: default_sync_sleep_secs(),
            submission_sleep_secs: default_submission_sleep_secs(),
            max_iterations: None,
            dry_run: false,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
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
    /// 2. Config file (if provided)
    /// 3. Default values
    pub fn load(config_file: Option<PathBuf>) -> Result<Self> {
        let mut figment = Figment::from(Serialized::defaults(Config::default()));

        // Add config file if provided
        if let Some(path) = config_file {
            figment = figment.merge(Toml::file(&path))
        }

        // Add environment variables with RELAYER_ prefix
        figment = figment.merge(Env::prefixed("RELAYER_").split("_"));

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

        // Validate secret key file exists
        if !self.near.secret_key_path.exists() {
            return Err(color_eyre::eyre::eyre!(
                "Secret key file does not exist: {}",
                self.near.secret_key_path.display()
            ));
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
        tracing::info!("  Secret key path: {}", self.near.secret_key_path.display());
        tracing::info!(
            "  Update interval: {} epochs",
            self.relayer.update_interval_epochs
        );
        tracing::info!("  Batch size: {} headers", self.relayer.headers_batch_size);
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
        assert_eq!(
            config.beacon.endpoint,
            "http://unstable.sepolia.beacon-api.nimbus.team"
        );
        assert_eq!(config.relayer.headers_batch_size, 32);
        assert_eq!(config.logging.level, "info");
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
