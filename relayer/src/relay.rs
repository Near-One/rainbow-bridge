use std::time::Duration;

use crate::constants::protocol::SLOTS_PER_EPOCH;
use crate::{BeaconClient, ContractClient, ExecutionClient};
use color_eyre::Result;
use eth2_utility::types::ClientMode;
use near_crypto::{InMemorySigner, SecretKey};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::config::Config;

#[derive(Debug)]
pub enum RelayResult {
    Submitted,
    Skipped,
    Failed(color_eyre::Report),
}

pub struct EthRelayer {
    beacon_client: BeaconClient,
    execution_client: ExecutionClient,
    near_client: ContractClient,
    config: Config,
}

impl EthRelayer {
    pub async fn new(config: Config) -> Result<Self> {
        let beacon_client = BeaconClient::new(&config.beacon.endpoint)?;
        let execution_client = ExecutionClient::from_config(&config.execution)?;
        let near_client = Self::create_near_client(&config).await?;

        Ok(Self {
            beacon_client,
            execution_client,
            near_client,
            config,
        })
    }

    /// Allow injecting clients for tests
    pub fn with_clients(
        beacon_client: BeaconClient,
        execution_client: ExecutionClient,
        near_client: ContractClient,
        config: Config,
    ) -> Self {
        Self {
            beacon_client,
            execution_client,
            near_client,
            config,
        }
    }

    async fn create_near_client(config: &Config) -> Result<ContractClient> {
        let secret_key: SecretKey = config.near.secret_key.trim().parse()?;
        let (contract_account_id, signer_account_id) = config.parse_near_accounts()?;
        let signer = InMemorySigner::from_secret_key(signer_account_id, secret_key);
        let client = near_fetch::Client::new(&config.near.endpoint);

        Ok(ContractClient::new(contract_account_id, signer, client, config.relayer.clone()))
    }

    pub async fn run(&self) -> Result<()> {
        info!("🚀 Starting ETH to NEAR relayer");

        for iteration in 1.. {
            if self
                .config
                .relayer
                .max_iterations
                .is_some_and(|max| iteration > max)
            {
                info!("Reached maximum iterations, stopping");
                break;
            }

            info!("=== Relay Loop {} ===", iteration);

            let result = self.run_iteration().await;
            let sleep_secs = match &result {
                RelayResult::Submitted => {
                    info!("✅ Operation completed");
                    self.config.relayer.submission_sleep_secs
                }
                RelayResult::Skipped => {
                    info!("⏭️ No work to do");
                    self.config.relayer.sync_sleep_secs
                }
                RelayResult::Failed(e) => {
                    error!("❌ Error: {}", e);
                    self.config.relayer.sync_sleep_secs
                }
            };

            sleep(Duration::from_secs(sleep_secs)).await;
        }
        Ok(())
    }

    pub async fn run_job(&self) -> Result<()> {
        info!("🚀 Starting ETH to NEAR relayer job (single execution)");

        let result = self.run_iteration().await;
        match &result {
            RelayResult::Submitted => {
                info!("✅ Job completed successfully");
            }
            RelayResult::Skipped => {
                info!("⏭️ Job completed, no work to do");
            }
            RelayResult::Failed(e) => {
                error!("❌ Job failed: {}", e);
                return Err(color_eyre::eyre::eyre!("Job execution failed: {}", e));
            }
        }

        Ok(())
    }

    async fn run_iteration(&self) -> RelayResult {
        // Early return pattern - convert all errors to RelayResult::Error
        let mode = match self.get_mode_if_synced().await {
            Ok(mode) => mode,
            Err(e) => return RelayResult::Failed(e),
        };

        let result = match mode {
            ClientMode::SubmitLightClientUpdate => {
                info!("📡 Light Client Update Mode");
                self.try_submit_light_client_update().await
            }
            ClientMode::SubmitHeader => {
                info!("🔗 Submit Header Mode");
                self.try_submit_headers().await
            }
        };

        result.unwrap_or_else(RelayResult::Failed)
    }

    async fn get_mode_if_synced(&self) -> Result<ClientMode> {
        if self.beacon_client.is_syncing().await? {
            info!("⏳ Beacon node is syncing, waiting...");
            return Err(color_eyre::eyre::eyre!("Beacon node is syncing"));
        }
        info!("✅ All clients are synchronized");
        self.near_client.get_client_mode().await
    }

    async fn try_submit_light_client_update(&self) -> Result<RelayResult> {
        if self.config.relayer.dry_run {
            info!("🧪 DRY RUN: Would submit light client update");
            return Ok(RelayResult::Submitted);
        }

        let near_slot = self.near_client.get_finalized_beacon_block_slot().await?;
        let eth_slot = self.beacon_client.get_last_finalized_slot().await?;
        info!("Finalized slots - NEAR: {}, ETH: {}", near_slot, eth_slot);

        if !self.should_update(near_slot, eth_slot) {
            debug!(
                "No update needed: ETH slot {} <= NEAR slot {} or insufficient difference",
                eth_slot, near_slot
            );
            return Ok(RelayResult::Skipped);
        }

        let update = self.fetch_update_for_slots(near_slot, eth_slot).await?;
        self.near_client.submit_light_client_update(update).await?;

        Ok(RelayResult::Submitted)
    }

    async fn try_submit_headers(&self) -> Result<RelayResult> {
        if self.config.relayer.dry_run {
            info!("🧪 DRY RUN: Would submit execution headers");
            return Ok(RelayResult::Submitted);
        }

        let last_block = self.near_client.get_last_block_number().await?;
        let max_block = self.get_max_block().await?;

        if max_block <= last_block {
            debug!(
                "No new blocks to submit (last: {}, max: {})",
                last_block, max_block
            );
            return Ok(RelayResult::Skipped);
        }

        let start_block = last_block + 1;
        let capped_max_block = std::cmp::min(
            max_block,
            start_block + self.config.relayer.max_headers_per_period as u64 - 1,
        );
        
        info!(
            "Fetching {} execution headers for blocks {} to {} (capped from {})",
            capped_max_block - last_block,
            start_block,
            capped_max_block,
            max_block
        );

        let mut headers = self
            .execution_client
            .fetch_block_range(start_block..=capped_max_block)
            .await?;

        if headers.is_empty() {
            warn!(
                "No headers fetched for range {}..={}",
                start_block, capped_max_block
            );
            return Ok(RelayResult::Skipped);
        }

        info!("Fetched {} headers, submitting to NEAR", headers.len());
        headers.reverse();
        self.near_client.submit_execution_headers(&headers).await?;

        Ok(RelayResult::Submitted)
    }

    async fn fetch_update_for_slots(
        &self,
        near_slot: u64,
        eth_slot: u64,
    ) -> Result<eth_types::eth2::LightClientUpdate> {
        let near_period = BeaconClient::get_period_for_slot(near_slot);
        let eth_period = BeaconClient::get_period_for_slot(eth_slot);

        if eth_period == near_period {
            self.beacon_client.fetch_finality_update().await
        } else {
            self.beacon_client
                .fetch_period_update(near_period + 1)
                .await
        }
    }

    fn should_update(&self, near_slot: u64, eth_slot: u64) -> bool {
        eth_slot > near_slot
            && (eth_slot - near_slot)
                >= (SLOTS_PER_EPOCH * self.config.relayer.update_interval_epochs)
    }

    async fn get_max_block(&self) -> Result<u64> {
        match self.near_client.get_unfinalized_tail_block_number().await? {
            Some(tail) => Ok(tail - 1),
            None => {
                let slot = self.near_client.get_finalized_beacon_block_slot().await?;
                self.beacon_client.get_block_number_for_slot(slot).await
            }
        }
    }
}
