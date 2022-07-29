use crate::{EthClient, PAUSE_SUBMIT_UPDATE};
use admin_controlled::AdminControlled;
use eth2_utility::consensus::*;
use eth2_utility::types::InitInput;
use eth_types::eth2::*;
use eth_types::{BlockHeader, H256};
use hex::FromHex;
use lazy_static::lazy_static;
use near_sdk::test_utils::{accounts, VMContextBuilder};
use near_sdk::{testing_env, AccountId, VMConfig};
use tree_hash::TreeHash;

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

struct TestContext<'a> {
    contract: EthClient,
    headers: &'a Vec<BlockHeader>,
    updates: &'a Vec<LightClientUpdate>,
}

struct InitOptions {
    pub validate_updates: bool,
    pub verify_bls_signatures: bool,
    pub hashes_gc_threshold: u64,
    pub max_submitted_blocks_by_account: u32,
    pub trusted_signer: Option<AccountId>,
}

fn get_kiln_test_context(init_options: Option<InitOptions>) -> TestContext<'static> {
    const NETWORK: &str = "kiln";
    lazy_static! {
        static ref INIT_UPDATE: LightClientUpdate =
            read_client_updates(NETWORK.to_string(), 99, 99)[0].clone();
        static ref UPDATES: Vec<LightClientUpdate> =
            read_client_updates(NETWORK.to_string(), 100, 101);
        static ref HEADERS: Vec<BlockHeader> = read_headers(format!(
            "./src/data/{}/execution_blocks_{}_{}.json",
            NETWORK, 766535, 769622
        ));
    };

    let init_options = init_options.unwrap_or(InitOptions {
        validate_updates: true,
        verify_bls_signatures: true,
        hashes_gc_threshold: 51000,
        max_submitted_blocks_by_account: 7000,
        trusted_signer: None,
    });

    let contract = EthClient::init(InitInput {
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
    });

    assert_eq!(contract.last_block_number(), HEADERS[0].number);

    TestContext {
        contract,
        headers: &HEADERS,
        updates: &UPDATES,
    }
}

#[test]
pub fn test_submit_update_periods_100_101() {
    let submitter = accounts(0);
    let TestContext {
        mut contract,
        headers,
        updates,
    } = get_kiln_test_context(None);
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter, attached_deposit: contract.min_storage_balance_for_submitter());

    contract.register_submitter();
    for header in headers.iter().skip(1) {
        contract.submit_execution_header(header.clone());
        assert!(contract.is_known_execution_header(header.calculate_hash()));
        assert!(contract.block_hash_safe(header.number).is_none());
    }

    contract.submit_beacon_chain_light_client_update(updates[1].clone());

    for header in headers.iter().skip(1) {
        assert!(!contract.is_known_execution_header(header.calculate_hash()));
        assert!(
            contract.block_hash_safe(header.number).is_some(),
            "Execution block hash is not finalized: {:?}",
            header.calculate_hash()
        );
    }

    assert_eq!(contract.last_block_number(), headers.last().unwrap().number);
    contract.unregister_submitter();
}

#[test]
pub fn test_gc_headers() {
    let submitter = accounts(0);
    let TestContext {
        mut contract,
        headers,
        updates,
    } = get_kiln_test_context(Some(InitOptions {
        validate_updates: true,
        verify_bls_signatures: true,
        hashes_gc_threshold: 500,
        max_submitted_blocks_by_account: 7000,
        trusted_signer: None,
    }));
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter, attached_deposit: contract.min_storage_balance_for_submitter());

    contract.register_submitter();
    for header in headers.iter().skip(1) {
        contract.submit_execution_header(header.clone());
        assert!(contract.is_known_execution_header(header.calculate_hash()));
        assert!(contract.block_hash_safe(header.number).is_none());
    }

    contract.submit_beacon_chain_light_client_update(updates[1].clone());

    for header in headers.iter().skip(1).rev().take(500) {
        assert!(!contract.is_known_execution_header(header.calculate_hash()));
        assert!(
            contract.block_hash_safe(header.number).is_some(),
            "Execution block hash is not finalized: {:?}",
            header.calculate_hash()
        );
    }

    assert_eq!(contract.last_block_number(), headers.last().unwrap().number);

    for header in headers.iter().skip(1).rev().skip(500) {
        assert!(!contract.is_known_execution_header(header.calculate_hash()));
        assert!(
            contract.block_hash_safe(header.number).is_none(),
            "Execution block hash was not removed: {:?}",
            header.calculate_hash()
        );
    }
}

#[test]
#[should_panic(expected = "exhausted the limit of blocks")]
pub fn test_panic_on_exhausted_submit_limit() {
    let submitter = accounts(0);
    let TestContext {
        mut contract,
        headers,
        updates,
    } = get_kiln_test_context(Some(InitOptions {
        validate_updates: true,
        verify_bls_signatures: true,
        hashes_gc_threshold: 7100,
        max_submitted_blocks_by_account: 100,
        trusted_signer: None,
    }));
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter, attached_deposit: contract.min_storage_balance_for_submitter());

    contract.register_submitter();
    for header in headers.iter().skip(1) {
        contract.submit_execution_header(header.clone());
        assert!(contract.is_known_execution_header(header.calculate_hash()));
        assert!(contract.block_hash_safe(header.number).is_none());
    }

    contract.submit_beacon_chain_light_client_update(updates[1].clone());
}

#[test]
#[should_panic(expected = "only trusted_signer can update the client")]
pub fn test_trusted_signer() {
    let trusted_signer = accounts(1);
    let TestContext {
        mut contract,
        headers: _,
        updates,
    } = get_kiln_test_context(Some(InitOptions {
        validate_updates: true,
        verify_bls_signatures: true,
        hashes_gc_threshold: 7100,
        max_submitted_blocks_by_account: 100,
        trusted_signer: Some(trusted_signer),
    }));
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
    contract.submit_beacon_chain_light_client_update(updates[1].clone());
}

#[test]
#[should_panic(expected = "Invalid finality proof")]
pub fn test_panic_on_invalid_finality_proof() {
    let TestContext {
        mut contract,
        headers: _,
        updates,
    } = get_kiln_test_context(None);
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
    let mut update = updates[1].clone();
    update.finality_update.finality_branch[5] = H256::from(vec![]);
    contract.submit_beacon_chain_light_client_update(update);
}

#[test]
#[should_panic(expected = "Invalid execution block hash proof")]
pub fn test_panic_on_invalid_execution_block_proof() {
    let TestContext {
        mut contract,
        headers: _,
        updates,
    } = get_kiln_test_context(None);
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
    let mut update = updates[1].clone();
    update.finality_update.header_update.execution_hash_branch[5] = H256::from(vec![]);
    contract.submit_beacon_chain_light_client_update(update);
}

#[test]
#[should_panic(expected = "The acceptable update periods are")]
pub fn test_panic_on_skip_update_period() {
    let TestContext {
        mut contract,
        headers: _,
        updates,
    } = get_kiln_test_context(None);
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
    let mut update = updates[1].clone();
    update.finality_update.header_update.beacon_header.slot =
        update.signature_slot + EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH * 10;
    contract.submit_beacon_chain_light_client_update(update);
}

#[test]
#[should_panic(expected = "Unknown execution block hash")]
pub fn test_panic_on_submit_update_with_missing_execution_blocks() {
    let submitter = accounts(0);
    let TestContext {
        mut contract,
        headers,
        updates,
    } = get_kiln_test_context(None);
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter, attached_deposit: contract.min_storage_balance_for_submitter());

    contract.register_submitter();
    for header in headers.iter().skip(1).take(5) {
        contract.submit_execution_header(header.clone());
        assert!(contract.is_known_execution_header(header.calculate_hash()));
        assert!(contract.block_hash_safe(header.number).is_none());
    }

    contract.submit_beacon_chain_light_client_update(updates[1].clone());
}

#[test]
#[should_panic(expected = "paused")]
pub fn test_panic_on_submit_update_paused() {
    let TestContext {
        mut contract,
        headers: _,
        updates,
    } = get_kiln_test_context(None);
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0), current_account_id: accounts(0));
    contract.set_paused(PAUSE_SUBMIT_UPDATE);
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(1), current_account_id: accounts(0));
    contract.submit_beacon_chain_light_client_update(updates[1].clone());
}

#[test]
#[should_panic(expected = "The active header slot number should be higher than the finalized slot")]
pub fn test_panic_on_submit_outdated_update() {
    let submitter = accounts(0);
    let TestContext {
        mut contract,
        headers: _,
        updates,
    } = get_kiln_test_context(None);
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter, attached_deposit: contract.min_storage_balance_for_submitter());

    contract.submit_beacon_chain_light_client_update(updates[0].clone());
}

#[test]
#[should_panic(expected = "Parent should be submitted first")]
pub fn test_panic_on_submit_blocks_with_unknown_parent() {
    let submitter = accounts(0);
    let TestContext {
        mut contract,
        headers,
        updates: _,
    } = get_kiln_test_context(None);
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter, attached_deposit: contract.min_storage_balance_for_submitter());

    assert_eq!(contract.last_block_number(), headers[0].number);
    contract.register_submitter();

    for header in &headers[1..5] {
        contract.submit_execution_header(header.clone());
        assert!(contract.is_known_execution_header(header.calculate_hash()));
        assert!(contract.block_hash_safe(header.number).is_none());
    }

    // Skip 6th block
    for header in &headers[7..8] {
        contract.submit_execution_header(header.clone());
        assert!(contract.is_known_execution_header(header.calculate_hash()));
        assert!(contract.block_hash_safe(header.number).is_none());
    }
}

#[test]
#[should_panic(expected = "Can't unregister the account with used storage")]
pub fn test_panic_on_unregister_submitter() {
    let submitter = accounts(0);
    let TestContext {
        mut contract,
        headers,
        updates: _,
    } = get_kiln_test_context(None);
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter, attached_deposit: contract.min_storage_balance_for_submitter());

    assert_eq!(contract.last_block_number(), headers[0].number);

    contract.register_submitter();
    for header in &headers[1..5] {
        contract.submit_execution_header(header.clone());
        assert!(contract.is_known_execution_header(header.calculate_hash()));
        assert!(contract.block_hash_safe(header.number).is_none());
    }

    contract.unregister_submitter();
}

#[test]
#[should_panic(expected = "is not registered")]
pub fn test_panic_on_skipping_register_submitter() {
    let submitter = accounts(0);
    let TestContext {
        mut contract,
        headers,
        updates: _,
    } = get_kiln_test_context(None);
    set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter, attached_deposit: contract.min_storage_balance_for_submitter());

    assert_eq!(contract.last_block_number(), headers[0].number);

    for header in &headers[1..5] {
        contract.submit_execution_header(header.clone());
        assert!(contract.is_known_execution_header(header.calculate_hash()));
        assert!(contract.block_hash_safe(header.number).is_none());
    }
}
