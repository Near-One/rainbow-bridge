use borsh::{BorshDeserialize, BorshSerialize};
use eth_types::H256;
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

/// Client mode enum matching the TypeScript implementation
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[repr(u8)]
pub enum ClientMode {
    SubmitLightClientUpdate,
    SubmitHeader,
}

/// Placeholder for LightClientState - you'll need to define this based on your contract
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct LightClientState {
    // Add your actual fields here
    // This is just a placeholder
    pub placeholder: bool,
}

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
}

/// Builder pattern for easier contract instantiation
pub struct NearContractBuilder {
    rpc_url: Option<String>,
    contract_account_id: Option<AccountId>,
    signer: Option<Signer>,
}

impl NearContractBuilder {
    pub fn new() -> Self {
        Self {
            rpc_url: None,
            contract_account_id: None,
            signer: None,
        }
    }

    pub fn rpc_url(mut self, url: impl Into<String>) -> Self {
        self.rpc_url = Some(url.into());
        self
    }

    pub fn contract_account_id(mut self, account_id: AccountId) -> Self {
        self.contract_account_id = Some(account_id);
        self
    }

    pub fn signer(mut self, signer: Signer) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Build for testnet with default settings
    pub fn testnet(mut self) -> Self {
        self.rpc_url = Some("https://rpc.testnet.near.org".to_string());
        if self.contract_account_id.is_none() {
            self.contract_account_id = Some("client-eth2.sepolia.testnet".parse().unwrap());
        }
        self
    }

    /// Build for mainnet with default settings
    pub fn mainnet(mut self) -> Self {
        self.rpc_url = Some("https://rpc.mainnet.near.org".to_string());
        self
    }

    pub fn build(self) -> Result<NearContract> {
        let rpc_url = self
            .rpc_url
            .ok_or(NearContractError::BuilderFieldMissing { field: "rpc_url" })?;
        let contract_account_id =
            self.contract_account_id
                .ok_or(NearContractError::BuilderFieldMissing {
                    field: "contract_account_id",
                })?;
        let signer = self
            .signer
            .ok_or(NearContractError::BuilderFieldMissing { field: "signer" })?;

        let client = Client::new(&rpc_url);

        Ok(NearContract::new(contract_account_id, signer, client))
    }
}

impl Default for NearContractBuilder {
    fn default() -> Self {
        Self::new()
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

        let near_contract = NearContractBuilder::new()
            .testnet()
            .signer(signer.into())
            .build()?;

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

    #[tokio::test]
    async fn test_builder_validation() -> Result<()> {
        // Test missing signer
        let result = NearContractBuilder::new().testnet().build();

        match result {
            Err(NearContractError::BuilderFieldMissing { field: "signer" }) => {}
            _ => panic!("Expected BuilderFieldMissing error for signer"),
        }

        // Test missing RPC URL
        let signer = InMemorySigner::from_secret_key(
            "test.testnet".parse()?,
            SecretKey::from_random(KeyType::ED25519),
        );

        let result = NearContractBuilder::new()
            .contract_account_id("test.testnet".parse()?)
            .signer(signer.into())
            .build();

        match result {
            Err(NearContractError::BuilderFieldMissing { field: "rpc_url" }) => {}
            _ => panic!("Expected BuilderFieldMissing error for rpc_url"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_builder_pattern() -> Result<()> {
        let signer = InMemorySigner::from_secret_key(
            "test.testnet".parse()?,
            SecretKey::from_random(KeyType::ED25519),
        );

        // Test custom configuration
        let _contract = NearContractBuilder::new()
            .rpc_url("https://custom-rpc.example.com")
            .contract_account_id("custom-contract.testnet".parse()?)
            .signer(signer.into())
            .build()?;

        // Test testnet defaults
        let signer2 = InMemorySigner::from_secret_key(
            "test2.testnet".parse()?,
            SecretKey::from_random(KeyType::ED25519),
        );

        let _contract2 = NearContractBuilder::new()
            .testnet()
            .signer(signer2.into())
            .build()?;

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
