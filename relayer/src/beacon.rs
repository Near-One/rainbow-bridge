use crate::error::{LightClientError, Result};
use eth_types::eth2::LightClientUpdate as BorshLightClientUpdate;
use eth2::{BeaconNodeHttpClient, Timeouts};
use sensitive_url::SensitiveUrl;
use std::time::Duration;
use types::{
    ForkVersionedResponse, Hash256, LightClientFinalityUpdate, LightClientOptimisticUpdate,
    LightClientUpdate as LighthouseLightClientUpdate, MainnetEthSpec,
};

/// Service that uses Lighthouse's HTTP client to interact with beacon node APIs
pub struct BeaconLightClientService {
    client: BeaconNodeHttpClient,
}

impl BeaconLightClientService {
    /// Create a new service pointing to a beacon node HTTP API
    pub fn new(beacon_url: &str) -> Result<Self> {
        let url = SensitiveUrl::parse(beacon_url)?;

        // Configure timeouts - adjust as needed
        let timeouts = Timeouts::set_all(Duration::from_secs(30));

        let client = BeaconNodeHttpClient::new(url, timeouts);

        Ok(Self { client })
    }

    /// Fetch light client update for a specific period using Lighthouse's client
    pub async fn fetch_period_update(&self, period: u64) -> Result<BorshLightClientUpdate> {
        let updates: Vec<ForkVersionedResponse<LighthouseLightClientUpdate<MainnetEthSpec>>> = self
            .client
            .get_beacon_light_client_updates(period, 1)
            .await?
            .ok_or_else(|| {
                LightClientError::NotFound(format!(
                    "No light client updates found for period {}",
                    period
                ))
            })?;

        let update = updates.into_iter().next().ok_or_else(|| {
            LightClientError::NotFound(format!(
                "No light client update found for period {}",
                period
            ))
        })?;
        println!("Fork name: {}", update.version.unwrap());

        let v = serde_json::to_value(&update.data).unwrap();
        let custom_update: BorshLightClientUpdate = serde_json::from_value(v).unwrap();

        Ok(custom_update)
    }

    /// Fetch latest finality update
    pub async fn fetch_finality_update(&self) -> Result<BorshLightClientUpdate> {
        let finality_update: Option<
            ForkVersionedResponse<LightClientFinalityUpdate<MainnetEthSpec>>,
        > = self
            .client
            .get_beacon_light_client_finality_update()
            .await?;

        let json_str = serde_json::to_string(&finality_update.unwrap().data).unwrap();
        let custom_update: BorshLightClientUpdate = serde_json::from_str(&json_str).unwrap();

        Ok(custom_update)
    }

    /// Fetch optimistic update
    pub async fn fetch_optimistic_update(&self) -> Result<BorshLightClientUpdate> {
        let optimistic_update: Option<
            ForkVersionedResponse<LightClientOptimisticUpdate<MainnetEthSpec>>,
        > = self
            .client
            .get_beacon_light_client_optimistic_update()
            .await?;

        let json_str = serde_json::to_string(&optimistic_update.unwrap().data).unwrap();
        let custom_update: BorshLightClientUpdate = serde_json::from_str(&json_str).unwrap();

        Ok(custom_update)
    }

    /// Fetch light client bootstrap for a block root
    pub async fn fetch_bootstrap(
        &self,
        block_root: Hash256,
    ) -> Result<types::LightClientBootstrap<MainnetEthSpec>> {
        let bootstrap = self.client.get_light_client_bootstrap(block_root).await?;

        Ok(bootstrap.unwrap().data)
    }

    /// Get the last finalized slot
    pub async fn get_last_finalized_slot(&self) -> Result<u64> {
        let finality_checkpoints = self
            .client
            .get_beacon_states_finality_checkpoints(eth2::types::StateId::Head)
            .await?;

        let finalized_epoch = finality_checkpoints.unwrap().data.finalized.epoch;
        Ok(finalized_epoch.as_u64() * 32) // Convert epoch to slot
    }

    /// Check if the beacon node is syncing
    pub async fn is_syncing(&self) -> Result<bool> {
        let sync_status = self.client.get_node_syncing().await?;
        Ok(sync_status.data.is_syncing)
    }

    /// Calculate sync committee period for a given slot
    pub fn get_period_for_slot(slot: u64) -> u64 {
        const SLOTS_PER_EPOCH: u64 = 32;
        const EPOCHS_PER_PERIOD: u64 = 256;
        slot / (SLOTS_PER_EPOCH * EPOCHS_PER_PERIOD)
    }
}
