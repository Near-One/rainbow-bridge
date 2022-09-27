use eth_types::eth2::{LightClientState, LightClientUpdate};
use eth_types::{BlockHeader, H256};
use near_primitives::views::FinalExecutionOutcomeView;
use near_primitives::types::AccountId;
use near_sdk::Balance;
use std::error::Error;
use std::vec::Vec;

/// Interface for using Ethereum Light Client
pub trait EthClientContractTrait {
    /// Returns the last submitted slot by this relay
    fn get_last_submitted_slot(&self) -> u64;

    /// Check if block with the execution block hash is known in Eth Client
    fn is_known_block(&self, execution_block_hash: &H256) -> Result<bool, Box<dyn Error>>;

    /// Submit the Light Client Update to Eth Client. Returns the Transaction Status
    fn send_light_client_update(
        &mut self,
        light_client_update: LightClientUpdate,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn Error>>;

    /// Get finalized beacon block hash from Eth Client
    fn get_finalized_beacon_block_hash(&self) -> Result<H256, Box<dyn Error>>;

    /// Get finalized beacon block slot from Eth Client
    fn get_finalized_beacon_block_slot(&self) -> Result<u64, Box<dyn Error>>;

    /// Send headers to Eth Client.
    ///
    /// # Arguments
    ///
    /// * `headers` - the list of headers for submission to Eth Client
    /// * `end_slot` - the slot of the last header in list
    fn send_headers(
        &mut self,
        headers: &Vec<BlockHeader>,
        end_slot: u64,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn std::error::Error>>;

    /// Get the minimum deposit for registration new relay
    fn get_min_deposit(&self) -> Result<Balance, Box<dyn Error>>;

    /// Register current relay on the Eth Client
    fn register_submitter(&self) -> Result<FinalExecutionOutcomeView, Box<dyn Error>>;

    /// Check if the relay is registered on the Eth Client
    fn is_submitter_registered(&self, account_id: Option<AccountId>) -> Result<bool, Box<dyn Error>>;

    /// Get the Light Client State of the Eth Client
    fn get_light_client_state(&self) -> Result<LightClientState, Box<dyn Error>>;

    /// Get number of unfinalized blocks submitted by current relay and currently stored on contract
    fn get_num_of_submitted_blocks_by_account(&self) -> Result<u32, Box<dyn Error>>;

    /// Get max possible number of unfinalized blocks which can be stored on contract for one account
    fn get_max_submitted_blocks_by_account(&self) -> Result<u32, Box<dyn Error>>;
}
