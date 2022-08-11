use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    // endpoint to a full node of Eth2 Beacon chain with Light Client API
    pub beacon_endpoint: String,

    // endpoint for the Ethereum full node, which supports Eth1 RPC API
    pub eth1_endpoint: String,

    // the max number of headers submitted in one batch to eth client
    pub total_submit_headers: u32,

    // endpoint for a full node on the NEAR chain
    pub near_endpoint: String,

    // Account id from which relay make requests
    pub signer_account_id: String,

    // Path to the file with a secret key for signer account
    pub path_to_signer_secret_key: String,

    // Account id for eth client contract on NEAR
    pub contract_account_id: String,

    // The Ethereum network name (main, kiln)
    pub network: String,

    // Contract type (near, dao, file)
    pub contract_type: String,

    // Frequency of submission light client updates. Once in N epochs.
    pub light_client_updates_submission_frequency_in_epochs: i64,

    // maximum gap in slots between submitting light client update
    pub max_blocks_for_finalization: u64,

    // NEAR network name (mainnet, testnet)
    pub near_network_id: String,

    // Account id for DAO on NEAR
    pub dao_contract_account_id: Option<String>,

    // Path to dir for output submitted light client updates and execution blocks
    pub output_dir: Option<String>,
}

impl Config {
    pub fn load_from_toml(path: PathBuf) -> Self {
        let mut config = std::fs::File::open(path).unwrap();
        let mut content = String::new();
        config.read_to_string(&mut content).unwrap();
        toml::from_str(content.as_str()).unwrap()
    }
}
