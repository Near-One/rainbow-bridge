use alloy::{
    network::Ethereum,
    primitives::U64,
    providers::{Provider, RootProvider},
    rpc::{client::RpcClient, types::Block},
};
use eth_types::BlockHeader;
use std::ops::RangeInclusive;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutionClientError {
    #[error("Alloy provider error: {0}")]
    Provider(#[from] alloy::transports::RpcError<alloy::transports::TransportErrorKind>),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Invalid block range: start block {start} must be <= end block {end}")]
    InvalidRange { start: u64, end: u64 },
    #[error("Tokio join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("JSON serialization error: {0}")]
    JsonSerde(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, ExecutionClientError>;

/// ExecutionClient provides methods for interacting with Ethereum execution layer
pub struct ExecutionClient {
    provider: RootProvider<Ethereum>,
    client: RpcClient,
}

impl ExecutionClient {
    /// Creates a new ExecutionClient with the given RPC URL
    pub fn new(rpc_url: &str) -> Result<Self> {
        let client = RpcClient::new_http(rpc_url.parse()?);
        let provider = RootProvider::new(client.clone());

        Ok(Self { provider, client })
    }

    /// Creates a new ExecutionClient for Sepolia testnet
    pub fn sepolia() -> Result<Self> {
        Self::new("https://ethereum-sepolia-rpc.publicnode.com")
    }

    /// Converts an alloy Block to our BlockHeader type
    fn convert_block_to_header(&self, block: Block) -> Result<BlockHeader> {
        let v = serde_json::to_value(&block.header.inner)?;
        let header: BlockHeader = serde_json::from_value(v)?;
        Ok(header)
    }

    /// Fetches a range of blocks using batch requests for efficiency
    ///
    /// # Arguments
    /// * `range` - A range of block numbers (inclusive on both ends)
    ///
    /// # Returns
    /// A vector of BlockHeader objects in the same order as the range
    pub async fn fetch_block_range(&self, range: RangeInclusive<u64>) -> Result<Vec<BlockHeader>> {
        let start_block = *range.start();
        let end_block = *range.end();

        if start_block > end_block {
            return Err(ExecutionClientError::InvalidRange {
                start: start_block,
                end: end_block,
            });
        }

        // Use batch requests for all range
        let mut batch = self.client.new_batch();

        // Add all block requests to the batch
        let mut futures = Vec::new();
        for block_number in range {
            let block_number_param = U64::from(block_number);
            let future = batch
                .add_call("eth_getBlockByNumber", &(block_number_param, false))?
                .map_resp(|resp: Option<Block>| resp);
            futures.push(future);
        }

        // Send the batch request
        batch.send().await?;

        // Collect all results and convert to BlockHeader
        let mut headers = Vec::new();
        for future in futures {
            match future.await? {
                Some(block) => {
                    let header = self.convert_block_to_header(block)?;
                    headers.push(header);
                }
                None => {} // Skip missing blocks
            }
        }

        Ok(headers)
    }

    /// Fetches the latest block number
    pub async fn get_latest_block_number(&self) -> Result<u64> {
        let block_number = self.provider.get_block_number().await?;
        Ok(block_number)
    }

    /// Fetches a range of recent blocks (last N blocks)
    pub async fn fetch_recent_blocks(&self, count: u64) -> Result<Vec<BlockHeader>> {
        let latest = self.get_latest_block_number().await?;
        let start = latest.saturating_sub(count - 1);
        self.fetch_block_range(start..=latest).await
    }

    /// Fetches a single block header by number
    pub async fn fetch_block_header(&self, block_number: u64) -> Result<Option<BlockHeader>> {
        let headers = self.fetch_block_range(block_number..=block_number).await?;
        Ok(headers.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = ExecutionClient::sepolia();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_range() {
        let client = ExecutionClient::sepolia().unwrap();
        let result = client.fetch_block_range(100..=50).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionClientError::InvalidRange { start, end } => {
                assert_eq!(start, 100);
                assert_eq!(end, 50);
            }
            _ => panic!("Expected InvalidRange error"),
        }
    }

    #[tokio::test]
    async fn test_single_block_fetch() {
        let client = ExecutionClient::sepolia().unwrap();
        let header = client.fetch_block_header(8440252).await.unwrap();
        assert!(header.is_some());

        let header = header.unwrap();
        println!("Block hash: {:#?}", header.calculate_hash());
        println!("Parent hash: {:#?}", header.parent_hash);
    }

    #[tokio::test]
    async fn test_block_range_and_hash_chain() {
        let client = ExecutionClient::sepolia().unwrap();
        let headers = client.fetch_block_range(8440252..=8440255).await.unwrap();

        println!("Fetched {} block headers:", headers.len());
        assert_eq!(headers.len(), 4);

        // Verify that each block's parent hash equals the previous block's calculated hash
        for i in 1..headers.len() {
            let current_block = &headers[i];
            let previous_block = &headers[i - 1];

            let previous_calculated_hash = previous_block.calculate_hash();
            let current_parent_hash = current_block.parent_hash;

            println!(
                "Block {}: parent_hash = {:#?}, previous_calculated = {:#?}",
                i, current_parent_hash, previous_calculated_hash
            );

            assert_eq!(
                current_parent_hash, previous_calculated_hash,
                "Block {} parent hash should equal previous block's calculated hash",
                i
            );
        }

        println!("✅ Hash chain verification passed!");
    }

    #[tokio::test]
    async fn test_recent_blocks() {
        let client = ExecutionClient::sepolia().unwrap();
        let headers = client.fetch_recent_blocks(3).await.unwrap();

        assert_eq!(headers.len(), 3);
        println!("Fetched {} recent blocks", headers.len());

        // Verify the blocks are in ascending order
        for i in 1..headers.len() {
            assert!(headers[i].number > headers[i - 1].number);
        }
    }

    #[tokio::test]
    async fn test_hash_calculation_consistency() {
        let client = ExecutionClient::sepolia().unwrap();
        let headers = client.fetch_block_range(8440250..=8440252).await.unwrap();

        for header in &headers {
            let calculated_hash = header.calculate_hash();
            println!(
                "Block {}: calculated hash = {:#?}",
                header.number, calculated_hash
            );

            // The hash should be consistent across multiple calculations
            assert_eq!(calculated_hash, header.calculate_hash());
        }
    }
}
