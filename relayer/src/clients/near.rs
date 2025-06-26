use borsh::BorshDeserialize;
use color_eyre::{Result, eyre::Context};
use eth_types::{
    BlockHeader, H256,
    eth2::{LightClientState, LightClientUpdate},
};
use eth2_utility::types::{ClientMode, InitInput};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use near_crypto::Signer;
use near_fetch::ops::Function;
use near_fetch::{Client, ops::MAX_GAS};
use near_gas::NearGas;
use near_primitives::types::AccountId;
use std::fmt::Write;
use tracing::info;

use crate::config::RelayerConfig;

/// NEAR contract client for Ethereum light client operations
#[derive(Clone)]
pub struct ContractClient {
    eth_light_client_account_id: AccountId,
    signer: Signer,
    client: Client,
    relayer_config: RelayerConfig,
}

impl ContractClient {
    /// Create a new NEAR contract client instance
    pub fn new(
        eth_light_client_account_id: AccountId,
        signer: Signer,
        client: Client,
        relayer_config: RelayerConfig,
    ) -> Self {
        Self {
            eth_light_client_account_id,
            signer,
            client,
            relayer_config,
        }
    }

    /// Helper method for contract view calls with proper error handling
    async fn call_contract_view<T>(&self, method_name: &str) -> Result<T>
    where
        T: BorshDeserialize,
    {
        let result = self
            .client
            .view(&self.eth_light_client_account_id, method_name)
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
            .view(&self.eth_light_client_account_id, "block_hash_safe")
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
                &self.eth_light_client_account_id,
                "submit_beacon_chain_light_client_update",
            )
            .args_borsh(update)
            .gas(NearGas::from_tgas(100))
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

        let batched_headers: Vec<&[BlockHeader]> = headers
            .chunks(self.relayer_config.headers_batch_size)
            .collect::<Vec<_>>();

        let total_batches = batched_headers.len();
        let total_headers = headers.len();

        // Create enhanced progress bar only for multi-batch operations
        let progress_bar = if total_batches > 1 {
            let pb = ProgressBar::new(total_headers as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "Submitting headers {wide_bar:.green/yellow} {pos}/{len} ({per_sec}, ETA {eta})",
                )
                .unwrap()
                .with_key("per_sec", |state: &ProgressState, w:&mut dyn Write| write!(w, "{:.1} headers/s", state.per_sec()).unwrap())
                .progress_chars("█▉▊▋▌▍▎▏  "),
            );
            Some(pb)
        } else {
            None
        };

        for (batch_index, header_batch) in batched_headers.iter().enumerate() {
            let attached_gas_per_promise_in_batch = MAX_GAS.as_gas() / header_batch.len() as u64;

            let mut batch = self
                .client
                .batch(&self.signer, &self.eth_light_client_account_id);

            for header in *header_batch {
                let function = Function::new("submit_execution_header")
                    .args_borsh(header.clone())
                    .gas(NearGas::from_gas(attached_gas_per_promise_in_batch));
                batch = batch.call(function);
            }

            let _ = batch
                .retry_exponential(1000, 3)
                .transact_async()
                .await
                .wrap_err(format!(
                    "Failed to submit execution headers batch {} of {}",
                    batch_index + 1,
                    total_batches
                ))?;

            // Update progress bar
            if let Some(ref pb) = progress_bar {
                pb.inc(header_batch.len() as u64);
            }
        }

        // Finish progress bar with summary
        if let Some(pb) = progress_bar {
            pb.finish_with_message(format!(
                "✅ {} headers submitted in {} batches",
                total_headers, total_batches
            ));
        } else {
            info!("✅ {} headers submitted successfully", total_headers);
        }

        Ok(())
    }

    pub async fn init_contract(&self, init_input: InitInput) -> Result<()> {
        self.client
            .call(&self.signer, &self.eth_light_client_account_id, "init")
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
    pub fn eth_light_client_account_id(&self) -> &AccountId {
        &self.eth_light_client_account_id
    }

    /// Get a reference to the underlying client
    pub fn client(&self) -> &Client {
        &self.client
    }
}
