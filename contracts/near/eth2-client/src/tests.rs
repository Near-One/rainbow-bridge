use eth2_utility::types::InitInput;
use eth_types::eth2::*;
use eth_types::{BlockHeader, H256};
use hex::FromHex;
use near_sdk::test_utils::{accounts, VMContextBuilder};
use near_sdk::{testing_env, VMConfig};
use tree_hash::TreeHash;

use crate::EthClient;

macro_rules! inner_set_env {
    ($builder:ident) => {
        $builder
    };

    ($builder:ident, $key:ident:$value:expr $(,$key_tail:ident:$value_tail:expr)*) => {
        {
           $builder.$key($value.try_into().unwrap());
           inner_set_env!($builder $(,$key_tail:$value_tail)*)
        }
    };
}

macro_rules! set_env {
    ($($key:ident:$value:expr),* $(,)?) => {
        let mut builder = VMContextBuilder::new();
        let mut builder = &mut builder;
        builder = inner_set_env!(builder, $($key: $value),*);
        let mut vm_config = VMConfig::free();
        vm_config.limit_config.max_number_logs = u64::MAX;
        vm_config.limit_config.max_total_log_length = u64::MAX;
        testing_env!(builder.build(), vm_config);
    };
}

fn read_beacon_header(filename: String) -> BeaconBlockHeader {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(&filename)).unwrap()).unwrap()
}

fn read_headers(filename: String) -> Vec<BlockHeader> {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(&filename)).unwrap()).unwrap()
}

fn read_client_update(filename: String) -> LightClientUpdate {
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

#[test]
pub fn test_header_root() {
    let header = read_beacon_header(format!("./src/data/kiln/beacon_header_{}.json", 5000));
    assert_eq!(
        H256(header.tree_hash_root()),
        Vec::from_hex("c613fbf1a8e95c2aa0f76a5d226ee1dc057cce18b235803f50e7a1bde050d290")
            .unwrap()
            .into()
    );

    let header = read_beacon_header(format!("./src/data/mainnet/beacon_header_{}.json", 4100000));
    assert_eq!(
        H256(header.tree_hash_root()),
        Vec::from_hex("342ca1455e976f300cc96a209106bed2cbdf87243167fab61edc6e2250a0be6c")
            .unwrap()
            .into()
    );
}

#[test]
pub fn test_submit_update_periods_100_101() {
    let submitter = accounts(0);
    let min_storage_balance_for_submitter = 8 * near_sdk::ONE_NEAR;
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter, attached_deposit: min_storage_balance_for_submitter);

    let network: String = "kiln".to_string();
    let updates = read_client_updates(network.clone(), 99, 101);
    let mut headers = read_headers(format!(
        "./src/data/{}/execution_blocks_{}_{}.json",
        network, 766535, 769622
    ));

    let mut contract = EthClient::init(InitInput {
        network,
        finalized_execution_header: headers[0].clone(),
        finalized_beacon_header: updates[1].clone().finality_update.header_update.into(),
        current_sync_committee: updates[0]
            .clone()
            .sync_committee_update
            .as_ref()
            .unwrap()
            .next_sync_committee
            .clone(),
        next_sync_committee: updates[1]
            .sync_committee_update
            .as_ref()
            .unwrap()
            .next_sync_committee
            .clone(),
        validate_updates: true,
        verify_bls_signatures: true,
        hashes_gc_threshold: 51000,
        max_submitted_blocks_by_account: 7000,
        trusted_signer: None,
    });

    assert_eq!(contract.last_block_number(), headers[0].number);
    headers.remove(0);

    contract.register_submitter();
    for header in &headers {
        contract.submit_execution_header(header.clone());
        assert!(contract.is_known_execution_header(header.calculate_hash()));
        assert!(contract.block_hash_safe(header.number).is_none());
    }

    contract.submit_beacon_chain_light_client_update(updates[2].clone());

    for header in &headers {
        assert!(!contract.is_known_execution_header(header.calculate_hash()));
        assert!(
            contract.block_hash_safe(header.number).is_some(),
            "Execution block hash is not finalized: {:?}",
            header.calculate_hash()
        );
    }

    assert_eq!(contract.last_block_number(), headers.last().unwrap().number);
}
