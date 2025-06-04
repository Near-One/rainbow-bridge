use borsh::BorshDeserialize;
use color_eyre::{Result, eyre::Context};
use eth_types::{
    BlockHeader, H256,
    eth2::{LightClientState, LightClientUpdate},
};
use eth2_utility::types::{ClientMode, InitInput};
use near_crypto::Signer;
use near_fetch::Client;
use near_primitives::types::AccountId;

/// NEAR contract client for Ethereum light client operations
pub struct NearContract {
    contract_account_id: AccountId,
    signer: Signer,
    client: Client,
}

impl NearContract {
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
            .wrap_err_with(|| format!("Failed to call view method '{}'", method_name))?
            .borsh::<T>()
            .wrap_err_with(|| format!("Failed to deserialize result from '{}'", method_name))?;
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

    pub async fn submit_light_client_update(&self, update: LightClientUpdate) -> Result<()> {
        let result = self
            .client
            .call(
                &self.signer,
                &self.contract_account_id,
                "submit_beacon_chain_light_client_update",
            )
            .args_borsh(update)
            .transact()
            .await
            .wrap_err("Failed to submit light client update transaction")?;

        result
            .into_result()
            .wrap_err("Light client update transaction failed")?;

        println!("Light client update submitted successfully");
        Ok(())
    }

    pub async fn submit_execution_header(&self, header: BlockHeader) -> Result<()> {
        let result = self
            .client
            .call(
                &self.signer,
                &self.contract_account_id,
                "submit_execution_header",
            )
            .args_borsh(header)
            .transact()
            .await
            .wrap_err("Failed to submit execution header transaction")?;

        result
            .into_result()
            .wrap_err("Execution header submission failed")?;

        println!("Execution header submitted successfully");
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

        println!("Contract initialized successfully");
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
