use crate::beacon_rpc_client::{BeaconRPCClient, LightClientSnapshotWithProof};
use crate::config::Config;
use crate::eth1_rpc_client::Eth1RPCClient;
use contract_wrapper::eth_client_contract::EthClientContract;
use eth2_utility::consensus::{convert_branch, floorlog2, get_subtree_index};
use eth_types::eth2::ExtendedBeaconBlockHeader;
use eth_types::BlockHeader;
use log::info;
use std::{thread, time};
use tree_hash::TreeHash;

const CURRENT_SYNC_COMMITTEE_INDEX: u32 = 54;
const CURRENT_SYNC_COMMITTEE_TREE_DEPTH: u32 = floorlog2(CURRENT_SYNC_COMMITTEE_INDEX);
const CURRENT_SYNC_COMMITTEE_TREE_INDEX: u32 = get_subtree_index(CURRENT_SYNC_COMMITTEE_INDEX);

pub fn verify_light_client_snapshot(
    block_root: String,
    light_client_snapshot: &LightClientSnapshotWithProof,
) -> bool {
    if block_root
        != format!(
            "{:#x}",
            light_client_snapshot.beacon_header.tree_hash_root()
        )
    {
        return false;
    }
    let branch = convert_branch(&light_client_snapshot.current_sync_committee_branch);
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
    init_block_root: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(target: "relay", "=== Contract initialization ===");

    if init_block_root.is_empty() {
        return Err("init_block_root shouldn't be empty".into());
    }

    let beacon_rpc_client = BeaconRPCClient::new(
        &config.beacon_endpoint,
        config.eth_requests_timeout_seconds,
        config.state_requests_timeout_seconds,
    );
    let eth1_rpc_client = Eth1RPCClient::new(&config.eth1_endpoint);

    let light_client_update = beacon_rpc_client
        .get_finality_light_client_update_with_sync_commity_update()
        .unwrap();
    let block_id = format!(
        "{}",
        light_client_update
            .finality_update
            .header_update
            .beacon_header
            .slot
    );
    let finalized_header: ExtendedBeaconBlockHeader =
        ExtendedBeaconBlockHeader::from(light_client_update.finality_update.header_update);
    let finalized_body = beacon_rpc_client
        .get_beacon_block_body_for_block_id(&block_id)
        .unwrap();

    let finalized_execution_header: BlockHeader = eth1_rpc_client
        .get_block_header_by_number(
            finalized_body
                .execution_payload()
                .unwrap()
                .execution_payload
                .block_number,
        )
        .unwrap();
    let next_sync_committee = light_client_update
        .sync_committee_update
        .unwrap()
        .next_sync_committee;

    let light_client_snapshot = beacon_rpc_client
        .get_bootstrap(init_block_root.clone())
        .expect("Unable to fetch bootstrap state");
    if !verify_light_client_snapshot(init_block_root, &light_client_snapshot) {
        return Err("Invalid light client snapshot".into());
    }

    eth_client_contract.init_contract(
        config.network.to_string(),
        finalized_execution_header,
        finalized_header,
        light_client_snapshot.current_sync_committee,
        next_sync_committee,
    );

    thread::sleep(time::Duration::from_secs(30));
    Ok(())
}
