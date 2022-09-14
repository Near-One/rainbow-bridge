use crate::beacon_rpc_client::BeaconRPCClient;
use crate::config::Config;
use crate::eth1_rpc_client::Eth1RPCClient;
use crate::eth2near_relay::Eth2NearRelay;
use crate::init_contract::init_contract;
use crate::test_utils;
use contract_wrapper::eth_client_contract::EthClientContract;
use contract_wrapper::eth_client_contract_trait::EthClientContractTrait;
use contract_wrapper::sandbox_contract_wrapper::SandboxContractWrapper;
use eth_types::eth2::{ExtendedBeaconBlockHeader, LightClientUpdate, SyncCommittee};
use eth_types::BlockHeader;
use std::{thread, time};
use tokio::runtime::Runtime;
use tree_hash::TreeHash;
use workspaces::prelude::*;
use workspaces::{network::Sandbox, Account, Contract, Worker};

pub fn read_json_file_from_data_dir(file_name: &str) -> std::string::String {
    let mut json_file_path = std::env::current_exe().unwrap();
    json_file_path.pop();
    json_file_path.push("../../../data");
    json_file_path.push(file_name);

    std::fs::read_to_string(json_file_path).expect("Unable to read file")
}

pub fn init_contract_from_files(eth_client_contract: &mut EthClientContract) {
    const PATH_TO_CURRENT_SYNC_COMMITTEE: &str =
        "../contract_wrapper/data/next_sync_committee_kiln_period_133.json";
    const PATH_TO_NEXT_SYNC_COMMITTEE: &str =
        "../contract_wrapper/data/next_sync_committee_kiln_period_134.json";
    const NETWORK: &str = "kiln";
    const PATH_TO_EXECUTION_BLOCKS: &str =
        "../contract_wrapper/data/execution_block_headers_kiln_1099394-1099937.json";
    const PATH_TO_LIGHT_CLIENT_UPDATES: &str =
        "../contract_wrapper/data/light_client_updates_kiln_1099394-1099937.json";

    let execution_blocks: Vec<BlockHeader> = serde_json::from_str(
        &std::fs::read_to_string(PATH_TO_EXECUTION_BLOCKS).expect("Unable to read file"),
    )
    .unwrap();

    let light_client_updates: Vec<LightClientUpdate> = serde_json::from_str(
        &std::fs::read_to_string(PATH_TO_LIGHT_CLIENT_UPDATES).expect("Unable to read file"),
    )
    .unwrap();

    let current_sync_committee: SyncCommittee = serde_json::from_str(
        &std::fs::read_to_string(PATH_TO_CURRENT_SYNC_COMMITTEE).expect("Unable to read file"),
    )
    .unwrap();
    let next_sync_committee: SyncCommittee = serde_json::from_str(
        &std::fs::read_to_string(PATH_TO_NEXT_SYNC_COMMITTEE).expect("Unable to read file"),
    )
    .unwrap();

    let finalized_beacon_header = ExtendedBeaconBlockHeader::from(
        light_client_updates[0]
            .clone()
            .finality_update
            .header_update,
    );

    let finalized_hash = light_client_updates[0]
        .clone()
        .finality_update
        .header_update
        .execution_block_hash;
    let mut finalized_execution_header = None::<BlockHeader>;
    for header in &execution_blocks {
        if header.hash.unwrap() == finalized_hash {
            finalized_execution_header = Some(header.clone());
            break;
        }
    }

    eth_client_contract.init_contract(
        NETWORK.to_string(),
        finalized_execution_header.unwrap(),
        finalized_beacon_header,
        current_sync_committee,
        next_sync_committee,
    );
    thread::sleep(time::Duration::from_secs(30));
}

pub fn init_contract_from_specific_slot(
    eth_client_contract: &mut EthClientContract,
    finality_slot: u64,
) {
    const PATH_TO_CURRENT_SYNC_COMMITTEE: &str =
        "../contract_wrapper/data/next_sync_committee_kiln_period_133.json";
    const PATH_TO_NEXT_SYNC_COMMITTEE: &str =
        "../contract_wrapper/data/next_sync_committee_kiln_period_134.json";
    const NETWORK: &str = "kiln";
    const TIMEOUT_SECONDS: u64 = 30;
    const TIMEOUT_STATE_SECONDS: u64 = 1000;

    let current_sync_committee: SyncCommittee = serde_json::from_str(
        &std::fs::read_to_string(PATH_TO_CURRENT_SYNC_COMMITTEE).expect("Unable to read file"),
    )
    .unwrap();
    let next_sync_committee: SyncCommittee = serde_json::from_str(
        &std::fs::read_to_string(PATH_TO_NEXT_SYNC_COMMITTEE).expect("Unable to read file"),
    )
    .unwrap();

    let beacon_rpc_client = BeaconRPCClient::new("https://lodestar-kiln.chainsafe.io", TIMEOUT_SECONDS, TIMEOUT_STATE_SECONDS);
    let eth1_rpc_client = Eth1RPCClient::new("https://rpc.kiln.themerge.dev");

    let finality_header = beacon_rpc_client
        .get_beacon_block_header_for_block_id(&format!("{}", finality_slot))
        .unwrap();

    let finality_header = eth_types::eth2::BeaconBlockHeader {
        slot: finality_header.slot.as_u64(),
        proposer_index: finality_header.proposer_index,
        parent_root: finality_header.parent_root.into(),
        state_root: finality_header.state_root.into(),
        body_root: finality_header.body_root.into(),
    };

    let finalized_body = beacon_rpc_client
        .get_beacon_block_body_for_block_id(&format!("{}", finality_slot))
        .unwrap();

    let finalized_beacon_header = ExtendedBeaconBlockHeader {
        header: finality_header.clone(),
        beacon_block_root: eth_types::H256(finality_header.tree_hash_root()),
        execution_block_hash: finalized_body
            .execution_payload()
            .unwrap()
            .execution_payload
            .block_hash
            .into_root()
            .into(),
    };

    let finalized_execution_header: BlockHeader = eth1_rpc_client
        .get_block_header_by_number(
            finalized_body
                .execution_payload()
                .unwrap()
                .execution_payload
                .block_number,
        )
        .unwrap();

    eth_client_contract.init_contract(
        NETWORK.to_string(),
        finalized_execution_header,
        finalized_beacon_header,
        current_sync_committee,
        next_sync_committee,
    );

    thread::sleep(time::Duration::from_secs(30));
}

const WASM_FILEPATH: &str = "../../contracts/near/res/eth2_client.wasm";

fn create_contract() -> (Account, Contract, Worker<Sandbox>) {
    let rt = Runtime::new().unwrap();

    let worker = rt.block_on(workspaces::sandbox()).unwrap();
    let wasm = std::fs::read(WASM_FILEPATH).unwrap();
    let contract = rt.block_on(worker.dev_deploy(&wasm)).unwrap();

    // create accounts
    let owner = worker.root_account().unwrap();
    let relay_account = rt
        .block_on(
            owner
                .create_subaccount(&worker, "relay_account")
                .initial_balance(30 * near_sdk::ONE_NEAR)
                .transact(),
        )
        .unwrap()
        .into_result()
        .unwrap();

    (relay_account, contract, worker)
}

fn get_config() -> Config {
    Config {
        beacon_endpoint: "https://lodestar-kiln.chainsafe.io".to_string(),
        eth1_endpoint: "https://rpc.kiln.themerge.dev".to_string(),
        total_submit_headers: 8,
        near_endpoint: "NaN".to_string(),
        signer_account_id: "NaN".to_string(),
        path_to_signer_secret_key: "NaN".to_string(),
        contract_account_id: "NaN".to_string(),
        network: "kiln".to_string(),
        contract_type: "near".to_string(),
        light_client_updates_submission_frequency_in_epochs: 1,
        max_blocks_for_finalization: 5000,
        near_network_id: "testnet".to_string(),
        dao_contract_account_id: None,
        output_dir: None,
        path_to_attested_state: None,
        path_to_finality_state: None,
        eth_requests_timeout_seconds: 30,
        state_requests_timeout_seconds: 1000,
    }
}

pub fn get_client_contract(from_file: bool) -> Box<dyn EthClientContractTrait> {
    let (relay_account, contract, worker) = create_contract();
    let contract_wrapper = Box::new(SandboxContractWrapper::new(relay_account, contract, worker));
    let mut eth_client_contract = EthClientContract::new(contract_wrapper);

    let config = get_config();
    match from_file {
        true => test_utils::init_contract_from_files(&mut eth_client_contract),
        false => init_contract(&config, &mut eth_client_contract).unwrap(),
    };

    Box::new(eth_client_contract)
}

pub fn get_relay(enable_binsearch: bool, from_file: bool) -> Eth2NearRelay {
    let config = get_config();
    Eth2NearRelay::init(
        &config,
        get_client_contract(from_file),
        enable_binsearch,
        true,
        false,
    )
}

pub fn get_relay_with_update_from_file(
    enable_binsearch: bool,
    from_file: bool,
    next_sync_committee: bool,
) -> Eth2NearRelay {
    let mut config = get_config();
    config.path_to_attested_state =
        Some("../contract_wrapper/data/beacon_state_kiln_slot_1099459.json".to_string());

    if next_sync_committee {
        config.path_to_finality_state =
            Some("../contract_wrapper/data/beacon_state_kiln_slot_1099392.json".to_string());
    }

    Eth2NearRelay::init(
        &config,
        get_client_contract(from_file),
        enable_binsearch,
        true,
        false,
    )
}

pub fn get_relay_from_slot(enable_binsearch: bool, slot: u64) -> Eth2NearRelay {
    let config = get_config();

    let (relay_account, contract, worker) = create_contract();
    let contract_wrapper = Box::new(SandboxContractWrapper::new(relay_account, contract, worker));
    let mut eth_client_contract = EthClientContract::new(contract_wrapper);

    init_contract_from_specific_slot(&mut eth_client_contract, slot);

    Eth2NearRelay::init(
        &config,
        Box::new(eth_client_contract),
        enable_binsearch,
        true,
        false,
    )
}
