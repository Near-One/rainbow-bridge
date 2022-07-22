use eth_types::BlockHeader;
use eth_types::eth2::ExtendedBeaconBlockHeader;
use crate::beacon_rpc_client::BeaconRPCClient;
use crate::eth1_rpc_client::Eth1RPCClient;
use crate::eth_client_contract::EthClientContract;


pub fn init_contract(near_endpoint: &str, signer_account_id: &str, path_to_signer_secret_key: &str,
                     contract_account_id: &str, start_slot: u64, output_dir: &str,
                     beacon_rpc_endpoint: &str, eth1_rpc_endpoint: &str, network: &str) -> Result<(), Box<dyn std::error::Error>> {
    let eth_client_contract = EthClientContract::new(near_endpoint, signer_account_id, path_to_signer_secret_key, contract_account_id, start_slot, output_dir.to_string());
    let period = BeaconRPCClient::get_period_for_slot(start_slot);

    let beacon_rpc_client = BeaconRPCClient::new(&beacon_rpc_endpoint);
    let eth1_rpc_client = Eth1RPCClient::new(&eth1_rpc_endpoint);

    let light_client_update = beacon_rpc_client.get_finality_light_client_update_with_sync_commity_update().unwrap();
    let block_id = format!("{}", light_client_update.finality_update.header_update.header.slot);
    let finalized_header : ExtendedBeaconBlockHeader = ExtendedBeaconBlockHeader::from(light_client_update.finality_update.header_update);
    let finalized_body = beacon_rpc_client.get_beacon_block_body_for_block_id(&block_id).unwrap();

    let finalized_execution_header: BlockHeader = eth1_rpc_client.get_block_header_by_number(finalized_body.execution_payload().unwrap().execution_payload.block_number).unwrap();
    let next_sync_committee = light_client_update.sync_committee_update.unwrap().next_sync_committee;
    let prev_light_client_update = beacon_rpc_client.get_light_client_update(period - 1)?;
    let current_sync_committee = prev_light_client_update.sync_committee_update.unwrap().next_sync_committee;

    eth_client_contract.init_contract(network.to_string(), finalized_execution_header, finalized_header, current_sync_committee, next_sync_committee);
    Ok(())
}