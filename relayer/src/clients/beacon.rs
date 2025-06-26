use crate::constants::protocol::{EPOCHS_PER_PERIOD, SLOTS_PER_EPOCH};
use color_eyre::{Result, eyre::Context};
use eth_types::eth2::LightClientUpdate as BorshLightClientUpdate;
use eth2::{BeaconNodeHttpClient, Timeouts};
use sensitive_url::SensitiveUrl;
use std::time::Duration;
use types::{
    ExecPayload, ForkVersionedResponse, FullPayloadRef, LightClientFinalityUpdate,
    LightClientUpdate as LighthouseLightClientUpdate, MainnetEthSpec, Slot,
};

/// Service that uses Lighthouse's HTTP client to interact with beacon node APIs
pub struct BeaconClient {
    client: BeaconNodeHttpClient,
}

impl BeaconClient {
    /// Create a new service pointing to a beacon node HTTP API
    pub fn new(beacon_url: &str) -> Result<Self> {
        let url = SensitiveUrl::parse(beacon_url).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to parse beacon URL '{}': {:?}", beacon_url, e)
        })?;

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
            .await
            .map_err(|e| {
                color_eyre::eyre::eyre!(
                    "Failed to fetch light client updates for period {}: {:?}",
                    period,
                    e
                )
            })?
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("No light client updates found for period {}", period)
            })?;

        let update = updates.into_iter().next().ok_or_else(|| {
            color_eyre::eyre::eyre!("No light client update found for period {}", period)
        })?;

        let v = serde_json::to_value(&update.data)
            .wrap_err("Failed to serialize light client update to JSON")?;
        let custom_update: BorshLightClientUpdate = serde_json::from_value(v)
            .wrap_err("Failed to deserialize light client update from JSON")?;

        Ok(custom_update)
    }

    /// Fetch latest finality update
    pub async fn fetch_finality_update(&self) -> Result<BorshLightClientUpdate> {
        let finality_update: Option<
            ForkVersionedResponse<LightClientFinalityUpdate<MainnetEthSpec>>,
        > = self
            .client
            .get_beacon_light_client_finality_update()
            .await
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to fetch light client finality update: {:?}", e)
            })?;

        let finality_data = finality_update
            .ok_or_else(|| color_eyre::eyre::eyre!("No finality update available"))?
            .data;

        let json_str = serde_json::to_string(&finality_data)
            .wrap_err("Failed to serialize finality update to JSON")?;
        let custom_update: BorshLightClientUpdate = serde_json::from_str(&json_str)
            .wrap_err("Failed to deserialize finality update from JSON")?;

        Ok(custom_update)
    }

    /// Get the last finalized slot
    pub async fn get_last_finalized_slot(&self) -> Result<u64> {
        let finality_checkpoints = self
            .client
            .get_beacon_states_finality_checkpoints(eth2::types::StateId::Head)
            .await
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to fetch finality checkpoints: {:?}", e)
            })?;

        let finalized_epoch = finality_checkpoints
            .ok_or_else(|| color_eyre::eyre::eyre!("No finality checkpoints available"))?
            .data
            .finalized
            .epoch;

        Ok(finalized_epoch.as_u64() * 32) // Convert epoch to slot
    }

    /// Check if the beacon node is syncing
    pub async fn is_syncing(&self) -> Result<bool> {
        let sync_status =
            self.client.get_node_syncing().await.map_err(|e| {
                color_eyre::eyre::eyre!("Failed to fetch node sync status: {:?}", e)
            })?;

        Ok(sync_status.data.is_syncing)
    }

    /// Get execution block number for a given beacon slot
    pub async fn get_block_number_for_slot(&self, slot: u64) -> Result<u64> {
        // Get the beacon block for this slot
        let block = self
            .client
            .get_beacon_blocks(eth2::types::BlockId::Slot(Slot::new(slot)))
            .await
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to fetch beacon block for slot {}: {:?}", slot, e)
            })?
            .ok_or_else(|| color_eyre::eyre::eyre!("No beacon block found for slot {}", slot))?;

        // Extract execution block number from the execution payload
        let execution_payload: FullPayloadRef<'_, MainnetEthSpec> = block
            .data
            .message()
            .body()
            .execution_payload()
            .map_err(|e| {
                color_eyre::eyre::eyre!(
                    "Failed to get execution payload for slot {}: {:?}",
                    slot,
                    e
                )
            })?;

        Ok(execution_payload.block_number())
    }

    /// Calculate sync committee period for a given slot
    pub fn get_period_for_slot(slot: u64) -> u64 {
        slot / (SLOTS_PER_EPOCH * EPOCHS_PER_PERIOD)
    }
}
