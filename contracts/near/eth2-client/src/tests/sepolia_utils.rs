use crate::tests::utils::InitOptions;
use eth2_utility::types::InitInput;
use eth_types::eth2::{ExtendedBeaconBlockHeader, LightClientUpdate};
use eth_types::BlockHeader;
use lazy_static::lazy_static;
use near_sdk::AccountId;

pub fn read_sepolia_beacon_header(filename: &str) -> ExtendedBeaconBlockHeader {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(filename)).unwrap()).unwrap()
}

pub fn read_sepolia_headers(filename: &str) -> Vec<BlockHeader> {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(filename)).unwrap()).unwrap()
}

pub fn read_sepolia_updates(filename: &str) -> Vec<LightClientUpdate> {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(filename)).unwrap()).unwrap()
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
        // load the one header window:
        static ref HEADERS: Vec<Vec<BlockHeader>> = vec![
            read_sepolia_headers(
                "./src/data/sepolia/execution_blocks_8262175_8262224.json"
            ),
            // duplicate or stub the second window so indexing won’t panic:
            read_sepolia_headers(
                "./src/data/sepolia/execution_blocks_8262175_8262224.json"
            ),
        ];
        // load all three periods:
        static ref UPDATES: Vec<LightClientUpdate> = vec![
            read_sepolia_updates("./src/data/sepolia/light_client_update_period_919.json")[0].clone(),
            read_sepolia_updates("./src/data/sepolia/light_client_update_period_920.json")[0].clone(),
            read_sepolia_updates("./src/data/sepolia/light_client_update_period_921.json")[0].clone(),
        ];
        // load the finalized beacon header message:
        static ref BEACON: ExtendedBeaconBlockHeader = {
            let mut hdr: ExtendedBeaconBlockHeader = serde_json::from_reader(
                std::fs::File::open(
                    "./src/data/sepolia/beacon_header_7560768.json"
                ).unwrap()
            ).unwrap();
            hdr
        };
    };

    let opts = init_options.unwrap_or(InitOptions {
        validate_updates: true,
        verify_bls_signatures: true,
        hashes_gc_threshold: 51_000,
        trusted_signer: None,
    });

    let init_input = InitInput {
        network: NETWORK.to_string(),
        finalized_execution_header: HEADERS[0][0].clone(),
        finalized_beacon_header: BEACON.clone(),
        current_sync_committee: UPDATES[0]
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
        validate_updates: opts.validate_updates,
        verify_bls_signatures: opts.verify_bls_signatures,
        hashes_gc_threshold: opts.hashes_gc_threshold,
        trusted_signer: opts.trusted_signer.clone(),
    };

    (&HEADERS, &UPDATES, init_input)
}
