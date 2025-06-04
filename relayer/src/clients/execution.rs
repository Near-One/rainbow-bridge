use alloy::{
    network::Ethereum,
    primitives::U64,
    providers::{Provider, RootProvider},
    rpc::{client::RpcClient, types::Block},
};
use color_eyre::Result;
use eth_types::BlockHeader;
use indicatif::{ProgressBar, ProgressStyle};
use std::ops::RangeInclusive;

/// ExecutionClient provides methods for interacting with Ethereum execution layer
pub struct ExecutionClient {
    provider: RootProvider<Ethereum>,
    client: RpcClient,
    max_batch_size: usize,
}

impl ExecutionClient {
    /// Creates a new ExecutionClient from configuration
    pub fn from_config(config: &crate::config::ExecutionConfig) -> Result<Self> {
        let client = RpcClient::new_http(config.endpoint.parse()?);
        let provider = RootProvider::new(client.clone());

        Ok(Self {
            provider,
            client,
            max_batch_size: config.max_batch_size,
        })
    }

    /// Creates a new ExecutionClient for Sepolia testnet using default config
    pub fn sepolia() -> Result<Self> {
        let config = crate::config::ExecutionConfig::default();
        Self::from_config(&config)
    }

    /// Converts an alloy Block to our BlockHeader type
    fn convert_block_to_header(&self, block: Block) -> Result<BlockHeader> {
        let v = serde_json::to_value(&block.header.inner)?;
        let header: BlockHeader = serde_json::from_value(v)?;
        Ok(header)
    }

    /// Fetches a range of blocks using batch requests with automatic chunking
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
            return Err(color_eyre::eyre::eyre!(
                "Invalid block range: start block {} must be <= end block {}",
                start_block,
                end_block
            ));
        }

        let block_numbers: Vec<u64> = range.collect();
        let total_blocks = block_numbers.len();
        let chunks: Vec<_> = block_numbers.chunks(self.max_batch_size).collect();
        let total_chunks = chunks.len();

        // Create progress bar only for multi-chunk operations
        let progress_bar = if total_chunks > 1 {
            let pb = ProgressBar::new(total_chunks as u64);
            pb.set_message(format!("Fetching {} blocks", total_blocks));
            Some(pb)
        } else {
            None
        };

        let mut all_headers = Vec::new();

        for chunk in chunks {
            let mut batch = self.client.new_batch();
            let mut futures = Vec::new();

            // Add all block requests to the batch
            for &block_number in chunk {
                let block_number_param = U64::from(block_number);
                let future = batch
                    .add_call("eth_getBlockByNumber", &(block_number_param, false))?
                    .map_resp(|resp: Option<Block>| resp);
                futures.push(future);
            }

            // Send the batch request
            batch.send().await?;

            // Collect results and convert to BlockHeader
            for future in futures {
                match future.await? {
                    Some(block) => {
                        let header = self.convert_block_to_header(block)?;
                        all_headers.push(header);
                    }
                    None => {} // Skip missing blocks
                }
            }

            // Update progress bar
            if let Some(ref pb) = progress_bar {
                pb.inc(1);
            }
        }

        // Finish progress bar
        if let Some(pb) = progress_bar {
            pb.finish_with_message(format!("✅ {} blocks fetched", all_headers.len()));
        }

        Ok(all_headers)
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

    /// Gets the current max batch size
    pub fn max_batch_size(&self) -> usize {
        self.max_batch_size
    }

    /// Sets a new max batch size
    pub fn set_max_batch_size(&mut self, size: usize) {
        self.max_batch_size = size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation_with_batch_size() {
        let mut config = crate::config::ExecutionConfig::default();
        config.max_batch_size = 500;

        let client = ExecutionClient::from_config(&config);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().max_batch_size(), 500);
    }

    #[tokio::test]
    async fn test_large_batch_chunking() {
        let mut config = crate::config::ExecutionConfig::default();
        config.max_batch_size = 100;

        let client = ExecutionClient::from_config(&config).unwrap();

        // This should automatically chunk into multiple batches
        let headers = client.fetch_block_range(8440252..=8440352).await.unwrap(); // 101 blocks

        println!("Fetched {} headers with automatic chunking", headers.len());
        assert!(headers.len() == 101); // Some blocks might be missing
    }

    #[tokio::test]
    async fn test_small_vs_large_batch_size() {
        let mut config_small = crate::config::ExecutionConfig::default();
        config_small.max_batch_size = 5;
        let client_small = ExecutionClient::from_config(&config_small).unwrap();

        let mut config_large = crate::config::ExecutionConfig::default();
        config_large.max_batch_size = 1000;
        let client_large = ExecutionClient::from_config(&config_large).unwrap();

        let range = 8440252..=8440257; // 6 blocks

        let headers_chunked = client_small.fetch_block_range(range.clone()).await.unwrap();
        let headers_single = client_large.fetch_block_range(range).await.unwrap();

        // Results should be the same regardless of batching strategy
        assert_eq!(headers_chunked.len(), headers_single.len());

        // Verify the hashes match (order should be maintained)
        for (chunked, single) in headers_chunked.iter().zip(headers_single.iter()) {
            assert_eq!(chunked.calculate_hash(), single.calculate_hash());
        }
    }
}
