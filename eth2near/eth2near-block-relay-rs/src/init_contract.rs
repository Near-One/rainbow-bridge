use crate::beacon_rpc_client::BeaconRPCClient;
use crate::config::Config;
use crate::eth1_rpc_client::Eth1RPCClient;
use contract_wrapper::contract_wrapper_trait::ContractWrapper;
use contract_wrapper::eth_client_contract::EthClientContract;
use eth_types::eth2::ExtendedBeaconBlockHeader;
use eth_types::BlockHeader;
use log::info;
use std::{thread, time};

pub fn init_contract(
    config: &Config,
    eth_client_contract: &mut EthClientContract,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(target: "relay", "=== Contract initialization ===");

    let beacon_rpc_client = BeaconRPCClient::new(&config.beacon_endpoint);
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

    eth_client_contract.init_contract(
        config.network.to_string(),
        finalized_execution_header,
        finalized_header,
        current_sync_committee,
        next_sync_committee,
    );

    thread::sleep(time::Duration::from_secs(30));
    Ok(())
}
