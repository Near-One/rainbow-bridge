use crate::constants::app::DEFAULT_HEADER_BATCH_SIZE;
use borsh::BorshDeserialize;
use color_eyre::{Result, eyre::Context};
use eth_types::{
    BlockHeader, H256,
    eth2::{LightClientState, LightClientUpdate},
};
use eth2_utility::types::{ClientMode, InitInput};
use near_crypto::Signer;
use near_fetch::ops::Function;
use near_fetch::{Client, ops::MAX_GAS};
use near_gas::NearGas;
use near_primitives::types::AccountId;
use tracing::info;

/// NEAR contract client for Ethereum light client operations
#[derive(Clone)]
pub struct ContractClient {
    contract_account_id: AccountId,
    signer: Signer,
    client: Client,
}

impl ContractClient {
    /// Create a new NEAR contract client instance
    pub fn new(contract_account_id: AccountId, signer: Signer, client: Client) -> Self {
        Self {
            contract_account_id,
            signer,
            client,
        }
    }

    /// Helper method for contract view calls with proper error handling
    async fn call_contract_view<T>(&self, method_name: &str) -> Result<T>
    where
        T: BorshDeserialize,
    {
        let result = self
            .client
            .view(&self.contract_account_id, method_name)
            .await
            .wrap_err(format!("Failed to call view method '{}'", method_name))?
            .borsh::<T>()
            .wrap_err(format!(
                "Failed to deserialize result from '{}'",
                method_name
            ))?;
        Ok(result)
    }

    /// Get the finalized beacon block hash
    pub async fn get_finalized_beacon_block_hash(&self) -> Result<H256> {
        self.call_contract_view("finalized_beacon_block_root").await
    }

    /// Get the finalized beacon block slot
    pub async fn get_finalized_beacon_block_slot(&self) -> Result<u64> {
        self.call_contract_view("finalized_beacon_block_slot").await
    }

    /// Get the current client mode
    pub async fn get_client_mode(&self) -> Result<ClientMode> {
        self.call_contract_view("get_client_mode").await
    }

    /// Get the light client state
    pub async fn get_light_client_state(&self) -> Result<LightClientState> {
        self.call_contract_view("get_light_client_state").await
    }

    /// Get the last block number
    pub async fn get_last_block_number(&self) -> Result<u64> {
        self.call_contract_view("last_block_number").await
    }

    /// Get the unfinalized tail block number (returns None if not set)
    pub async fn get_unfinalized_tail_block_number(&self) -> Result<Option<u64>> {
        self.call_contract_view("get_unfinalized_tail_block_number")
            .await
    }

    /// Get block hash safely by block number
    pub async fn get_block_hash(&self, block_number: u64) -> Result<Option<H256>> {
        let result = self
            .client
            .view(&self.contract_account_id, "block_hash_safe")
            .args_borsh(block_number)
            .await
            .wrap_err(format!(
                "Failed to call view method 'block_hash_safe' with block number {}",
                block_number
            ))?
            .borsh::<Option<H256>>()
            .wrap_err(format!(
                "Failed to get block hash result for block number {}",
                block_number
            ))?;
        Ok(result)
    }

    pub async fn submit_light_client_update(&self, update: LightClientUpdate) -> Result<()> {
        self.client
            .call(
                &self.signer,
                &self.contract_account_id,
                "submit_beacon_chain_light_client_update",
            )
            .args_borsh(update)
            .retry_exponential(1000, 3)
            .transact()
            .await
            .wrap_err("Failed to send light client update transaction")?
            .into_result()
            .wrap_err("Failed to submit light client update")?;

        info!("Light client update submitted successfully");
        Ok(())
    }

    /// Submit multiple execution headers in a single batch transaction
    pub async fn submit_execution_headers(&self, headers: &[BlockHeader]) -> Result<()> {
        if headers.is_empty() {
            info!("No headers to submit");
            return Ok(());
        }

        let batched_headers = headers
            .chunks(DEFAULT_HEADER_BATCH_SIZE)
            .collect::<Vec<_>>();

        for header_batch in batched_headers {
            let attached_gas_per_promise_in_batch = MAX_GAS.as_gas() / header_batch.len() as u64;

            let mut batch = self.client.batch(&self.signer, &self.contract_account_id);

            for header in header_batch {
                let function = Function::new("submit_execution_header")
                    .args_borsh(header.clone())
                    .gas(NearGas::from_gas(attached_gas_per_promise_in_batch));
                batch = batch.call(function);
            }

            let result = batch
                .retry_exponential(1000, 3)
                .transact()
                .await
                .wrap_err("Failed to submit execution headers batch transaction")?;

            info!("Execution headers submitted successfully");
        }

        Ok(())
    }

    pub async fn init_contract(&self, init_input: InitInput) -> Result<()> {
        self.client
            .call(&self.signer, &self.contract_account_id, "init")
            .args_borsh(init_input)
            .transact()
            .await
            .wrap_err("Failed to send contract initialization transaction")?
            .into_result()
            .wrap_err("Contract initialization failed")?;

        info!("Contract initialized successfully");
        Ok(())
    }

    /// Get contract account ID
    pub fn contract_account_id(&self) -> &AccountId {
        &self.contract_account_id
    }

    /// Get a reference to the underlying client
    pub fn client(&self) -> &Client {
        &self.client
    }
}
