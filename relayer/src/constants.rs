/// Protocol-level constants for Ethereum consensus
pub mod protocol {
    /// Number of slots per epoch in Ethereum consensus
    pub const SLOTS_PER_EPOCH: u64 = 32;

    /// Number of epochs per sync committee period
    pub const EPOCHS_PER_PERIOD: u64 = 256;
}

/// Default configuration values for the relayer application
pub mod defaults {
    // Testnet network endpoints (Sepolia + NEAR testnet)
    pub const TESTNET_BEACON_ENDPOINT: &str = "http://unstable.sepolia.beacon-api.nimbus.team";
    pub const TESTNET_EXECUTION_ENDPOINT: &str = "https://ethereum-sepolia-rpc.publicnode.com";
    pub const TESTNET_NEAR_ENDPOINT: &str = "https://rpc.testnet.near.org";
    pub const TESTNET_ETH_LIGHT_CLIENT_ACCOUNT_ID: &str = "client-eth2.sepolia.testnet";
    pub const TESTNET_SIGNER_ACCOUNT_ID: &str = "relayer.testnet";

    // Mainnet network endpoints (Ethereum mainnet + NEAR mainnet)
    pub const MAINNET_BEACON_ENDPOINT: &str = "http://localhost:5052";
    pub const MAINNET_EXECUTION_ENDPOINT: &str = "https://eth.llamarpc.com";
    pub const MAINNET_NEAR_ENDPOINT: &str = "https://rpc.mainnet.near.org";
    pub const MAINNET_ETH_LIGHT_CLIENT_ACCOUNT_ID: &str = "client-eth2.near";
    pub const MAINNET_SIGNER_ACCOUNT_ID: &str = "relayer.near";

    // Legacy aliases (default to testnet)
    pub const BEACON_ENDPOINT: &str = TESTNET_BEACON_ENDPOINT;
    pub const EXECUTION_ENDPOINT: &str = TESTNET_EXECUTION_ENDPOINT;
    pub const NEAR_ENDPOINT: &str = TESTNET_NEAR_ENDPOINT;
    pub const ETH_LIGHT_CLIENT_ACCOUNT_ID: &str = TESTNET_ETH_LIGHT_CLIENT_ACCOUNT_ID;
    pub const SIGNER_ACCOUNT_ID: &str = TESTNET_SIGNER_ACCOUNT_ID;
    pub const SECRET_KEY_PATH: &str = "./keys/signer.txt";

    // Timeout configurations (in seconds)
    pub const TIMEOUT_SECS: u64 = 30;

    // Execution client settings
    pub const EXECUTION_BATCH_SIZE: usize = 500;

    // Relayer operation settings
    pub const UPDATE_INTERVAL_EPOCHS: u64 = 1;
    pub const HEADERS_BATCH_SIZE: usize = 100;
    pub const MAX_HEADERS_PER_LOOP: usize = 1000;
    pub const SYNC_SLEEP_SECS: u64 = 60;
    pub const SUBMISSION_SLEEP_SECS: u64 = 12;

    // Logging settings
    pub const LOG_LEVEL: &str = "info";

    // NEAR contract settings
    pub const HASHES_GC_THRESHOLD: u64 = 51_000;
}
