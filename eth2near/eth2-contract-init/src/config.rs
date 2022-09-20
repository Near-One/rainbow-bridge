use eth2_to_near_relay::near_rpc_client::NearRPCClient;
use reqwest::Url;
use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    // endpoint to a full node of Eth2 Beacon chain with Light Client API
    pub beacon_endpoint: String,

    // endpoint for the Ethereum full node, which supports Eth1 RPC API
    pub eth1_endpoint: String,

    // endpoint for a full node on the NEAR chain
    pub near_endpoint: String,

    // Account id from which relay make requests
    pub signer_account_id: String,

    // Path to the file with a secret key for signer account
    pub path_to_signer_secret_key: String,

    // Account id for eth client contract on NEAR
    pub contract_account_id: String,

    // The Ethereum network name (mainnet, kiln, ropsten, goerli)
    pub network: String,

    // NEAR network name (mainnet, testnet)
    pub near_network_id: String,

    // Path to dir for output submitted light client updates and execution blocks
    pub output_dir: Option<String>,

    // Timeout for ETH RPC requests in seconds
    pub eth_requests_timeout_seconds: u64,

    pub validate_updates: bool,

    pub verify_bls_signature: bool,

    pub hashes_gc_threshold: u64,

    pub max_submitted_blocks_by_account: u32,

    pub trusted_signature: Option<String>,
}

impl Config {
    pub fn load_from_toml(path: PathBuf) -> Self {
        let mut config = std::fs::File::open(path).unwrap();
        let mut content = String::new();
        config.read_to_string(&mut content).unwrap();
        let config = toml::from_str(content.as_str()).unwrap();

        Self::check_urls(&config);
        Self::check_account_id(&config);
        Self::check_network_types(&config);

        config
    }

    fn check_urls(&self) {
        // check `beacon_endpoint`
        Url::parse(&self.beacon_endpoint).unwrap();

        // check `eth1_endpoint`
        Url::parse(&self.eth1_endpoint).unwrap();

        // check `near_endpoint`
        Url::parse(&self.near_endpoint).unwrap();
    }

    fn check_account_id(&self) {
        let near_rpc_client = NearRPCClient::new(&self.near_endpoint);

        // check `signer_account_id`
        let _signer_account_id: near_sdk::AccountId = self.signer_account_id.parse().unwrap();
        if !near_rpc_client
            .check_account_exists(&self.signer_account_id)
            .unwrap()
        {
            panic!("Signer account id doesn't exist on NEAR network");
        }

        // check `trusted_signature`
        if let Some(trusted_signature) = self.trusted_signature.clone() {
            let _trusted_signature: near_sdk::AccountId = trusted_signature.parse().unwrap();
            if !near_rpc_client
                .check_account_exists(&trusted_signature)
                .unwrap()
            {
                panic!("Trusted signature doesn't exist on NEAR network");
            }
        }

        // check `contract_account_id`
        let _contract_account_id: near_sdk::AccountId = self.contract_account_id.parse().unwrap();
        if !near_rpc_client
            .check_account_exists(&self.contract_account_id)
            .unwrap()
        {
            panic!("Contract account id doesn't exist on NEAR network");
        }
    }

    fn check_network_types(&self) {
        // check `network`
        if !(self.network == "mainnet"
            || self.network == "kiln"
            || self.network == "ropsten"
            || self.network == "goerli")
        {
            panic!("Unknown network {}", self.network);
        }

        // check `near_network_id`
        if !(self.near_network_id == "mainnet" || self.near_network_id == "testnet") {
            panic!("Unknown near network id {}", self.near_network_id);
        }
    }
}
