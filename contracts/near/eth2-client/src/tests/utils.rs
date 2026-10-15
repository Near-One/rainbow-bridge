use std::str::FromStr;

use eth2_utility::consensus::Network;
use eth2_utility::types::InitInput;
use eth_types::eth2::*;
use eth_types::BlockHeader;
use lazy_static::lazy_static;
use near_sdk::AccountId;

pub fn read_beacon_header(filename: String) -> BeaconBlockHeader {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(&filename)).unwrap()).unwrap()
}

pub fn read_headers(filename: String) -> Vec<BlockHeader> {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(&filename)).unwrap()).unwrap()
}

pub fn read_client_update(filename: String) -> LightClientUpdate {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(&filename)).unwrap()).unwrap()
}

pub fn read_client_updates(
    network: String,
    start_period: u64,
    end_period: u64,
) -> Vec<LightClientUpdate> {
    let mut updates = vec![];
    for period_idx in start_period..=end_period {
        let client_update = read_client_update(format!(
            "./src/data/{}/light_client_update_period_{}.json",
            network, period_idx
        ));
        updates.push(client_update);
    }

    updates
}

pub struct InitOptions {
    pub validate_updates: bool,
    pub verify_bls_signatures: bool,
    pub hashes_gc_threshold: u64,
    pub trusted_signer: Option<AccountId>,
}

pub fn get_sepolia_test_data(
    init_options: Option<InitOptions>,
) -> (
    &'static Vec<Vec<BlockHeader>>,
    &'static Vec<LightClientUpdate>,
    InitInput,
) {
    const NETWORK: &str = "sepolia";
    lazy_static! {
        static ref INIT_UPDATE: LightClientUpdate =
            read_client_updates(NETWORK.to_string(), 925, 925)[0].clone();
        static ref UPDATES: Vec<LightClientUpdate> =
            read_client_updates(NETWORK.to_string(), 926, 928);
        static ref HEADERS: Vec<Vec<BlockHeader>> = vec![
            read_headers(format!(
                "./src/data/{}/execution_blocks_{}_{}.json",
                NETWORK, 8286935, 8295112
            )),
            read_headers(format!(
                "./src/data/{}/execution_blocks_{}_{}.json",
                NETWORK, 8295113, 8303246
            ))
        ];
    };

    let init_options = init_options.unwrap_or(InitOptions {
        validate_updates: true,
        verify_bls_signatures: true,
        hashes_gc_threshold: 51000,
        trusted_signer: None,
    });

    let init_input = InitInput {
        network: Network::from_str(NETWORK).unwrap(),
        finalized_execution_header: HEADERS[0][0].clone(),
        finalized_beacon_header: UPDATES[0].finalized_header.clone().into(),
        current_sync_committee: INIT_UPDATE
            .next_sync_committee
            .clone()
            .expect("Sync committee not found"),
        next_sync_committee: UPDATES[0]
            .next_sync_committee
            .clone()
            .expect("Sync committee not found"),
        validate_updates: init_options.validate_updates,
        verify_bls_signatures: init_options.verify_bls_signatures,
        hashes_gc_threshold: init_options.hashes_gc_threshold,
        trusted_signer: init_options.trusted_signer,
    };

    (&HEADERS, &UPDATES, init_input)
}

pub fn get_test_data(
    init_options: Option<InitOptions>,
) -> (
    &'static Vec<Vec<BlockHeader>>,
    &'static Vec<LightClientUpdate>,
    InitInput,
) {
    get_sepolia_test_data(init_options)
}

/// Sepolia data around the Gloas fork: the contract is initialized with the last Fulu period
/// (1378) and the update for the first Gloas period (1379) is submitted.
pub fn get_sepolia_gloas_test_data() -> (LightClientUpdate, InitInput) {
    const NETWORK: &str = "sepolia";
    let init_updates = read_client_updates(NETWORK.to_string(), 1377, 1378);
    let gloas_update = read_client_updates(NETWORK.to_string(), 1379, 1379)[0].clone();
    let finalized_execution_header = read_headers(format!(
        "./src/data/{}/execution_blocks_{}_{}.json",
        NETWORK, 11849735, 11849735
    ))[0]
        .clone();

    let init_input = InitInput {
        network: Network::from_str(NETWORK).unwrap(),
        finalized_execution_header,
        finalized_beacon_header: init_updates[1].finalized_header.clone().into(),
        current_sync_committee: init_updates[0]
            .next_sync_committee
            .clone()
            .expect("Sync committee not found"),
        next_sync_committee: init_updates[1]
            .next_sync_committee
            .clone()
            .expect("Sync committee not found"),
        validate_updates: true,
        verify_bls_signatures: true,
        hashes_gc_threshold: 51000,
        trusted_signer: None,
    };

    (gloas_update, init_input)
}
