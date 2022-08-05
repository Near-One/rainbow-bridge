use eth_types::eth2::{LightClientState, LightClientUpdate};
use eth_types::{BlockHeader, H256};
use near_primitives::hash::CryptoHash;
use near_sdk::Balance;
use std::error::Error;
use std::vec::Vec;

pub trait EthClientContractTrait {
    fn get_last_submitted_slot(&self) -> u64;
    fn is_known_block(&self, execution_block_hash: &H256) -> Result<bool, Box<dyn Error>>;
    fn send_light_client_update(
        &mut self,
        light_client_update: LightClientUpdate,
    ) -> Result<CryptoHash, Box<dyn Error>>;

    fn get_finalized_beacon_block_hash(&self) -> Result<H256, Box<dyn Error>>;
    fn get_finalized_beacon_block_slot(&self) -> Result<u64, Box<dyn Error>>;
    fn send_headers(
        &mut self,
        headers: &Vec<BlockHeader>,
        end_slot: u64,
    ) -> Result<CryptoHash, Box<dyn std::error::Error>>;

    fn get_min_deposit(&self) -> Result<Balance, Box<dyn Error>>;
    fn register_submitter(&self) -> Result<CryptoHash, Box<dyn Error>>;
    fn get_light_client_state(&self) -> Result<LightClientState, Box<dyn Error>>;
}
