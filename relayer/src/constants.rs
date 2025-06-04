/// Protocol-level constants for Ethereum consensus
pub mod protocol {
    /// Number of slots per epoch in Ethereum consensus
    pub const SLOTS_PER_EPOCH: u64 = 32;

    /// Number of epochs per sync committee period
    pub const EPOCHS_PER_PERIOD: u64 = 256;
}

/// Default configuration values for the relayer application
pub mod defaults {
    // Network endpoints
    pub const BEACON_ENDPOINT: &str = "http://unstable.sepolia.beacon-api.nimbus.team";
    pub const EXECUTION_ENDPOINT: &str = "https://ethereum-sepolia-rpc.publicnode.com";
    pub const NEAR_ENDPOINT: &str = "https://rpc.testnet.near.org";

    // Default account IDs (for example config only)
    pub const CONTRACT_ACCOUNT_ID: &str = "eth-client.testnet";
    pub const SIGNER_ACCOUNT_ID: &str = "relayer.testnet";
    pub const SECRET_KEY_PATH: &str = "./keys/signer.txt";

    // Timeout configurations (in seconds)
    pub const TIMEOUT_SECS: u64 = 30;

    // Execution client settings
    pub const EXECUTION_BATCH_SIZE: usize = 1000;

    // Relayer operation settings
    pub const UPDATE_INTERVAL_EPOCHS: u64 = 1;
    pub const HEADERS_BATCH_SIZE: usize = 32;
    pub const SYNC_SLEEP_SECS: u64 = 60;
    pub const SUBMISSION_SLEEP_SECS: u64 = 12;

    // Logging settings
    pub const LOG_LEVEL: &str = "info";

    // NEAR contract settings
    pub const HASHES_GC_THRESHOLD: u64 = 51_000;
}

/// Utility constants used throughout the application
pub mod app {
    /// Default batch size for processing headers in NEAR contract
    pub const DEFAULT_HEADER_BATCH_SIZE: usize = super::defaults::HEADERS_BATCH_SIZE;
}
