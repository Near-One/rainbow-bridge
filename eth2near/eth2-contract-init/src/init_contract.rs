use contract_wrapper::eth_client_contract::EthClientContract;
use eth2_to_near_relay::beacon_rpc_client::BeaconRPCClient;
use eth2_to_near_relay::eth1_rpc_client::Eth1RPCClient;
use eth_types::eth2::ExtendedBeaconBlockHeader;
use eth_types::BlockHeader;
use log::info;
use std::{thread, time};
use crate::config::Config;

pub fn init_contract(
    config: &Config,
    eth_client_contract: &mut EthClientContract,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(target: "relay", "=== Contract initialization ===");

    let beacon_rpc_client = BeaconRPCClient::new(
        &config.beacon_endpoint,
        config.eth_requests_timeout_seconds,
        config.eth_requests_timeout_seconds,
    );
    let eth1_rpc_client = Eth1RPCClient::new(&config.eth1_endpoint);

    let start_slot = beacon_rpc_client.get_last_finalized_slot_number().unwrap();
    let period = BeaconRPCClient::get_period_for_slot(start_slot.as_u64());

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
    let prev_light_client_update = beacon_rpc_client.get_light_client_update(period - 1)?;
    let current_sync_committee = prev_light_client_update
        .sync_committee_update
        .unwrap()
        .next_sync_committee;

    let mut trusted_signature: Option<near_primitives::types::AccountId> = Option::None;
    if let Some(trusted_signature_name) = config.trusted_signature.clone() {
        trusted_signature = Option::Some(trusted_signature_name.parse().unwrap());
    }

    if  config.near_network_id == "mainnet" {
        assert!(config.validate_updates, "The updates validation can't be disabled for mainnet");
        assert!(config.verify_bls_signatures || config.trusted_signer.is_some(), "The client can't be executed in the trustless mode without BLS sigs verification on Mainnet");
    }

    eth_client_contract.init_contract(
        config.network.to_string(),
        finalized_execution_header,
        finalized_header,
        current_sync_committee,
        next_sync_committee,
        config.validate_updates,
        config.verify_bls_signature,
        config.hashes_gc_threshold,
        config.max_submitted_blocks_by_account,
        trusted_signature,
    );

    thread::sleep(time::Duration::from_secs(30));
    Ok(())
}
