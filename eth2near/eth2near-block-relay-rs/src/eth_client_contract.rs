use eth_types::eth2::LightClientUpdate;
use crate::beacon_block_header_with_execution_data::BeaconBlockHeaderWithExecutionData;
use std::vec::Vec;

pub struct EthClientContract {}

impl EthClientContract {
    pub fn get_last_slot(&self) -> u64 {
        return 823661;
    }

    pub fn send_light_client_update(& mut self, light_client_update: LightClientUpdate) {}

    pub fn send_headers(& mut self, headers: Vec<BeaconBlockHeaderWithExecutionData>) {}
}