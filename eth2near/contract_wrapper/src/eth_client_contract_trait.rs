use eth_types::eth2::{LightClientState, LightClientUpdate};
use eth_types::{BlockHeader, H256};
use near_primitives::views::FinalExecutionOutcomeView;
use near_primitives::types::AccountId;
use near_sdk::Balance;
use std::error::Error;

/// Interface for using Ethereum Light Client
pub trait EthClientContractTrait {
    /// Returns the last submitted slot by this relay
    fn get_last_submitted_slot(&self) -> u64;

    /// Checks if the block with the execution block hash is known to Ethereum Light Client on NEAR
    fn is_known_block(&self, execution_block_hash: &H256) -> Result<bool, Box<dyn Error>>;

    /// Submits the Light Client Update to Ethereum Light Client on NEAR. Returns the final execution outcome or an error
    fn send_light_client_update(
        &mut self,
        light_client_update: LightClientUpdate,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn Error>>;

    /// Gets finalized beacon block hash from Ethereum Light Client on NEAR
    fn get_finalized_beacon_block_hash(&self) -> Result<H256, Box<dyn Error>>;

    /// Gets finalized beacon block slot from Ethereum Light Client on NEAR
    fn get_finalized_beacon_block_slot(&self) -> Result<u64, Box<dyn Error>>;

    /// Sends headers to Ethereum Light Client on NEAR. Returns final execution outcome or an error.
    ///
    /// # Arguments
    ///
    /// * `headers` - the list of headers for submission to Eth Client
    /// * `end_slot` - the slot of the last header in list
    fn send_headers(
        &mut self,
        headers: &[BlockHeader],
        end_slot: u64,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn std::error::Error>>;

    /// Gets the minimum required deposit for the registration of a new relay
    fn get_min_deposit(&self) -> Result<Balance, Box<dyn Error>>;

    /// Registers the current relay in the Ethereum Light Client on NEAR
    fn register_submitter(&self) -> Result<FinalExecutionOutcomeView, Box<dyn Error>>;

    /// Checks if the relay is registered in the Ethereum Light Client on NEAR
    fn is_submitter_registered(&self, account_id: Option<AccountId>) -> Result<bool, Box<dyn Error>>;

    /// Gets the Light Client State of the Ethereum Light Client on NEAR
    fn get_light_client_state(&self) -> Result<LightClientState, Box<dyn Error>>;
}
