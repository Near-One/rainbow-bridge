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
    pub max_submitted_blocks_by_account: u32,
    pub trusted_signer: Option<AccountId>,
}

pub fn get_goerli_test_data(
    init_options: Option<InitOptions>,
) -> (
    &'static Vec<BlockHeader>,
    &'static Vec<LightClientUpdate>,
    InitInput,
) {
    const NETWORK: &str = "goerli";
    lazy_static! {
        static ref INIT_UPDATE: LightClientUpdate =
            read_client_updates(NETWORK.to_string(), 632, 632)[0].clone();
        static ref UPDATES: Vec<LightClientUpdate> =
            read_client_updates(NETWORK.to_string(), 633, 634);
        static ref HEADERS: Vec<BlockHeader> = read_headers(format!(
            "./src/data/{}/execution_blocks_{}_{}.json",
            NETWORK, 8652100, 8661554
        ));
    };

    let init_options = init_options.unwrap_or(InitOptions {
        validate_updates: true,
        verify_bls_signatures: true,
        hashes_gc_threshold: 51000,
        max_submitted_blocks_by_account: 10000,
        trusted_signer: None,
    });

    let init_input = InitInput {
        network: NETWORK.to_string(),
        finalized_execution_header: HEADERS[0].clone(),
        finalized_beacon_header: UPDATES[0].clone().finality_update.header_update.into(),
        current_sync_committee: INIT_UPDATE
            .clone()
            .sync_committee_update
            .as_ref()
            .unwrap()
            .next_sync_committee
            .clone(),
        next_sync_committee: UPDATES[0]
            .sync_committee_update
            .as_ref()
            .unwrap()
            .next_sync_committee
            .clone(),
        validate_updates: init_options.validate_updates,
        verify_bls_signatures: init_options.verify_bls_signatures,
        hashes_gc_threshold: init_options.hashes_gc_threshold,
        max_submitted_blocks_by_account: init_options.max_submitted_blocks_by_account,
        trusted_signer: init_options.trusted_signer,
    };

    (&HEADERS, &UPDATES, init_input)
}

pub fn get_test_data(
    init_options: Option<InitOptions>,
) -> (
    &'static Vec<BlockHeader>,
    &'static Vec<LightClientUpdate>,
    InitInput,
) {
    get_goerli_test_data(init_options)
}
