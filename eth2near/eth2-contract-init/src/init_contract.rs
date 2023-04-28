use crate::config::Config;
use contract_wrapper::eth_client_contract::EthClientContract;
use contract_wrapper::near_network::NearNetwork;
use eth2_utility::consensus;
use eth_rpc_client::beacon_rpc_client::BeaconRPCClient;
use eth_rpc_client::eth1_rpc_client::Eth1RPCClient;
use eth_rpc_client::light_client_snapshot_with_proof::LightClientSnapshotWithProof;
use eth_types::eth2::ExtendedBeaconBlockHeader;
use eth_types::BlockHeader;
use log::info;
use std::{thread, time};
use tree_hash::TreeHash;
use types::{ExecutionPayload, MainnetEthSpec};

const CURRENT_SYNC_COMMITTEE_INDEX: u32 = 54;
const CURRENT_SYNC_COMMITTEE_TREE_DEPTH: u32 = consensus::floorlog2(CURRENT_SYNC_COMMITTEE_INDEX);
const CURRENT_SYNC_COMMITTEE_TREE_INDEX: u32 =
    consensus::get_subtree_index(CURRENT_SYNC_COMMITTEE_INDEX);

pub fn verify_light_client_snapshot(
    block_root: String,
    light_client_snapshot: &LightClientSnapshotWithProof,
) -> bool {
    let expected_block_root = format!(
        "{:#x}",
        light_client_snapshot.beacon_header.tree_hash_root()
    );

    if block_root != expected_block_root {
        return false;
    }

    let branch = consensus::convert_branch(&light_client_snapshot.current_sync_committee_branch);
    merkle_proof::verify_merkle_proof(
        light_client_snapshot
            .current_sync_committee
            .tree_hash_root(),
        &branch,
        CURRENT_SYNC_COMMITTEE_TREE_DEPTH.try_into().unwrap(),
        CURRENT_SYNC_COMMITTEE_TREE_INDEX.try_into().unwrap(),
        light_client_snapshot.beacon_header.state_root.0,
    )
}

pub fn init_contract(
    config: &Config,
    eth_client_contract: &mut EthClientContract,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(target: "relay", "=== Contract initialization ===");

    if let NearNetwork::Mainnet = config.near_network_id {
        assert!(
            config.validate_updates.unwrap_or(true),
            "The updates validation can't be disabled for mainnet"
        );
        assert!(config.verify_bls_signature.unwrap_or(false) || config.trusted_signer_account_id.is_some(), "The client can't be executed in the trustless mode without BLS sigs verification on Mainnet");
    }

    let beacon_rpc_client = BeaconRPCClient::new(
        &config.beacon_endpoint,
        config.eth_requests_timeout_seconds.unwrap_or(10),
        config.eth_requests_timeout_seconds.unwrap_or(10),
        Some(config.beacon_rpc_version.clone()),
    );
    let eth1_rpc_client = Eth1RPCClient::new(&config.eth1_endpoint);

    let last_period = BeaconRPCClient::get_period_for_slot(beacon_rpc_client
        .get_last_slot_number()
        .expect("Error on fetching last slot number")
        .as_u64());

    let light_client_update_with_next_sync_committee = beacon_rpc_client
        .get_light_client_update(last_period)
        .expect("Error on fetching finality light client update with sync committee update");
    let finality_light_client_update = beacon_rpc_client
        .get_finality_light_client_update()
        .expect("Error on fetching finality light client update");

    let finality_slot = finality_light_client_update
        .finality_update
        .header_update
        .beacon_header
        .slot;

    let block_id = format!("{}", finality_slot);

    let finalized_header: ExtendedBeaconBlockHeader =
        ExtendedBeaconBlockHeader::from(finality_light_client_update.finality_update.header_update);
    let finalized_body = beacon_rpc_client
        .get_beacon_block_body_for_block_id(&block_id)
        .expect("Error on fetching finalized body");

    let execution_payload: ExecutionPayload<MainnetEthSpec> = finalized_body
        .execution_payload()
        .expect("No execution payload in finalized body")
        .into();
    let finalized_execution_header: BlockHeader = eth1_rpc_client
        .get_block_header_by_number(execution_payload.block_number())
        .expect("Error on fetching finalized execution header");

    let next_sync_committee = light_client_update_with_next_sync_committee
        .sync_committee_update
        .expect("No sync_committee update in light client update")
        .next_sync_committee;

    let init_block_root = match config.init_block_root.clone() {
        None => beacon_rpc_client
            .get_checkpoint_root()
            .expect("Fail to get last checkpoint"),
        Some(init_block_str) => init_block_str,
    };

    let light_client_snapshot = beacon_rpc_client
        .get_bootstrap(init_block_root.clone())
        .expect("Unable to fetch bootstrap state");

    info!(target: "relay", "init_block_root: {}", init_block_root);

    if BeaconRPCClient::get_period_for_slot(light_client_snapshot.beacon_header.slot)
        != BeaconRPCClient::get_period_for_slot(finality_slot)
    {
        panic!("Period for init_block_root different from current period. Please use snapshot for current period");
    }

    if !verify_light_client_snapshot(init_block_root, &light_client_snapshot) {
        return Err("Invalid light client snapshot".into());
    }

    let mut trusted_signature: Option<near_primitives::types::AccountId> = Option::None;
    if let Some(trusted_signature_name) = config.trusted_signer_account_id.clone() {
        trusted_signature = Option::Some(
            trusted_signature_name
                .parse()
                .expect("Error on parsing trusted signature account"),
        );
    }

    eth_client_contract.init_contract(
        config.ethereum_network.clone(),
        finalized_execution_header,
        finalized_header,
        light_client_snapshot.current_sync_committee,
        next_sync_committee,
        config.validate_updates,
        config.verify_bls_signature,
        config.hashes_gc_threshold,
        trusted_signature,
    );

    thread::sleep(time::Duration::from_secs(30));
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config_for_tests::ConfigForTests;
    use crate::init_contract::init_contract;
    use contract_wrapper::eth_client_contract::EthClientContract;
    use contract_wrapper::eth_client_contract_trait::EthClientContractTrait;
    use contract_wrapper::near_network::NearNetwork;
    use contract_wrapper::sandbox_contract_wrapper::SandboxContractWrapper;
    use eth_rpc_client::beacon_rpc_client::{BeaconRPCClient, BeaconRPCVersion};
    use tokio::runtime::Runtime;
    use workspaces::{Account, Contract};

    const ONE_EPOCH_IN_SLOTS: u64 = 32;

    fn create_contract(config_for_test: &ConfigForTests) -> (Account, Contract) {
        let rt = Runtime::new().unwrap();
        let worker = rt.block_on(workspaces::sandbox()).unwrap();

        // create accounts
        let owner: Account = worker.root_account().unwrap();

        let wasm = std::fs::read(&config_for_test.wasm_filepath).unwrap();
        let contract = rt.block_on(owner.deploy(&wasm)).unwrap().unwrap();

        (owner, contract)
    }

    fn get_init_config(
        config_for_test: &ConfigForTests,
        eth_client_contract: &EthClientContract,
    ) -> crate::config::Config {
        return crate::config::Config {
            beacon_endpoint: config_for_test.beacon_endpoint.to_string(),
            eth1_endpoint: config_for_test.eth1_endpoint.to_string(),
            near_endpoint: "https://rpc.testnet.near.org".to_string(),
            signer_account_id: "NaN".to_string(),
            path_to_signer_secret_key: "NaN".to_string(),
            contract_account_id: "NaN".to_string(),
            ethereum_network: config_for_test.network_name.clone(),
            near_network_id: NearNetwork::Testnet,
            output_dir: None,
            eth_requests_timeout_seconds: Some(30),
            validate_updates: Some(true),
            verify_bls_signature: Some(false),
            hashes_gc_threshold: Some(51000),
            max_submitted_blocks_by_account: Some(8000),
            trusted_signer_account_id: Some(
                eth_client_contract.get_signer_account_id().to_string(),
            ),
            init_block_root: None,
            beacon_rpc_version: BeaconRPCVersion::V1_1,
        };
    }

    #[test]
    #[should_panic(expected = "The updates validation can't be disabled for mainnet")]
    fn test_init_contract_on_mainnet_without_validation() {
        let config_for_test =
            ConfigForTests::load_from_toml("config_for_tests.toml".try_into().unwrap());

        let (relay_account, contract) = create_contract(&config_for_test);
        let contract_wrapper = Box::new(SandboxContractWrapper::new(&relay_account, contract));

        let mut eth_client_contract = EthClientContract::new(contract_wrapper);
        let mut init_config = get_init_config(&config_for_test, &eth_client_contract);
        init_config.validate_updates = Some(false);
        init_config.near_network_id = NearNetwork::Mainnet;

        init_contract(&init_config, &mut eth_client_contract).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "The client can't be executed in the trustless mode without BLS sigs verification on Mainnet"
    )]
    fn test_init_contract_on_mainnet_without_trusted_signature() {
        let config_for_test =
            ConfigForTests::load_from_toml("config_for_tests.toml".try_into().unwrap());

        let (relay_account, contract) = create_contract(&config_for_test);
        let contract_wrapper = Box::new(SandboxContractWrapper::new(&relay_account, contract));

        let mut eth_client_contract = EthClientContract::new(contract_wrapper);
        let mut init_config = get_init_config(&config_for_test, &eth_client_contract);
        init_config.near_network_id = NearNetwork::Mainnet;
        init_config.trusted_signer_account_id = None;

        init_contract(&init_config, &mut eth_client_contract).unwrap();
    }

    #[test]
    fn test_sync_with_eth_after_init() {
        let config_for_test =
            ConfigForTests::load_from_toml("config_for_tests.toml".try_into().unwrap());

        let (relay_account, contract) = create_contract(&config_for_test);
        let contract_wrapper = Box::new(SandboxContractWrapper::new(&relay_account, contract));

        let mut eth_client_contract = EthClientContract::new(contract_wrapper);
        let mut init_config = get_init_config(&config_for_test, &eth_client_contract);
        init_config.beacon_rpc_version = BeaconRPCVersion::V1_5;

        init_contract(&init_config, &mut eth_client_contract).unwrap();

        let last_finalized_slot_eth_client = eth_client_contract
            .get_finalized_beacon_block_slot()
            .expect("Error on getting last finalized beacon block slot(Eth client)");

        let beacon_rpc_client = BeaconRPCClient::new(
            &init_config.beacon_endpoint,
            init_config.eth_requests_timeout_seconds.unwrap_or(10),
            init_config.eth_requests_timeout_seconds.unwrap_or(10),
            None,
        );

        let last_finalized_slot_eth_network = beacon_rpc_client
            .get_last_finalized_slot_number()
            .expect("Error on getting last finalized beacon block slot");

        const MAX_GAP_IN_EPOCH_BETWEEN_FINALIZED_SLOTS: u64 = 3;

        assert!(
            last_finalized_slot_eth_client
                + ONE_EPOCH_IN_SLOTS * MAX_GAP_IN_EPOCH_BETWEEN_FINALIZED_SLOTS
                >= last_finalized_slot_eth_network.as_u64()
        );
    }
}
