use borsh::BorshDeserialize;
use eth_types::{
    BlockHeader, H256,
    eth2::{LightClientState, LightClientUpdate},
};
use eth2_utility::types::{ClientMode, InitInput};
use near_crypto::Signer;
use near_fetch::Client;
use near_primitives::types::AccountId;
use thiserror::Error;

/// Errors that can occur when interacting with the NEAR contract
#[derive(Error, Debug)]
pub enum NearContractError {
    #[error("NEAR client error: {0}")]
    NearClient(#[from] near_fetch::Error),

    #[error("Borsh deserialization error: {0}")]
    BorshDeserialization(#[from] borsh::io::Error),

    #[error("Account ID parse error: {0}")]
    AccountIdParse(#[from] near_primitives::account::id::ParseAccountError),

    #[error("Secret key parse error: {0}")]
    SecretKeyParse(#[from] near_crypto::ParseKeyError),

    #[error("Builder error: {field} not set")]
    BuilderFieldMissing { field: &'static str },

    #[error("Contract call failed for method '{method}': {reason}")]
    ContractCallFailed { method: String, reason: String },
}

/// Result type for NEAR contract operations
pub type Result<T> = std::result::Result<T, NearContractError>;

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
            .map_err(|e| NearContractError::ContractCallFailed {
                method: method_name.to_string(),
                reason: e.to_string(),
            })?
            .borsh::<T>()
            .map_err(|e| NearContractError::ContractCallFailed {
                method: method_name.to_string(),
                reason: format!("Borsh deserialization failed: {}", e),
            })?;
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
            .map_err(|e| NearContractError::ContractCallFailed {
                method: "submit_beacon_chain_light_client_update".to_string(),
                reason: e.to_string(),
            })?;

        println!("Result: {:#?}", result);
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
            .map_err(|e| NearContractError::ContractCallFailed {
                method: "submit_execution_header".to_string(),
                reason: e.to_string(),
            })?;

        println!("Result: {:#?}", result);
        Ok(())
    }

    pub async fn init_contract(&self, init_input: InitInput) -> Result<()> {
        let result = self
            .client
            .call(&self.signer, &self.contract_account_id, "init")
            .args_borsh(init_input)
            .transact()
            .await
            .map_err(|e| NearContractError::ContractCallFailed {
                method: "init".to_string(),
                reason: e.to_string(),
            })?;

        println!("Result: {:#?}", result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_crypto::{InMemorySigner, KeyType, SecretKey};

    #[tokio::test]
    async fn test_integration_testnet() -> Result<()> {
        let signer = InMemorySigner::from_secret_key(
            "client-eth2.sepolia.testnet".parse()?,
            SecretKey::from_random(KeyType::ED25519),
        );

        let near_contract = NearContract::new(
            "client-eth2.sepolia.testnet".parse()?,
            signer,
            Client::new("https://rpc.sepolia.testnet.near.org"),
        );

        // Test finalized beacon block hash
        let hash = near_contract.get_finalized_beacon_block_hash().await?;
        println!("Finalized beacon block hash: {:#?}", hash);

        // Test other methods
        let slot = near_contract.get_finalized_beacon_block_slot().await?;
        println!("Finalized beacon block slot: {}", slot);

        let client_mode = near_contract.get_client_mode().await?;
        println!("Client mode: {:?}", client_mode);

        let last_block = near_contract.get_last_block_number().await?;
        println!("Last block number: {}", last_block);

        let unfinalized_tail = near_contract.get_unfinalized_tail_block_number().await?;
        println!("Unfinalized tail block number: {:?}", unfinalized_tail);

        Ok(())
    }
}

// Additional utility functions
impl NearContract {
    /// Get contract account ID
    pub fn contract_account_id(&self) -> &AccountId {
        &self.contract_account_id
    }

    /// Get a reference to the underlying client
    pub fn client(&self) -> &Client {
        &self.client
    }
}
