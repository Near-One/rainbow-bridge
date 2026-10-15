use crate::constants::protocol::{EPOCHS_PER_PERIOD, SLOTS_PER_EPOCH};
use color_eyre::{Result, eyre::Context};
use eth_types::eth2::LightClientUpdate as BorshLightClientUpdate;
use eth2::mixin::RequestAccept;
use eth2::types::Accept;
use eth2::{BeaconNodeHttpClient, Timeouts, Url};
use sensitive_url::SensitiveUrl;
use serde::{Deserialize, de::DeserializeOwned};
use std::time::Duration;

/// Versioned beacon API response (`{"version": ..., "data": ...}`)
#[derive(Deserialize)]
struct VersionedResponse<T> {
    data: T,
}

/// Service that uses Lighthouse's HTTP client to interact with beacon node APIs.
///
/// Light client data is requested as raw JSON and deserialized directly into our types:
/// Lighthouse types don't support Gloas light client data yet.
pub struct BeaconClient {
    client: BeaconNodeHttpClient,
    endpoint: Url,
}

impl BeaconClient {
    /// Create a new service pointing to a beacon node HTTP API
    pub fn new(beacon_url: &str) -> Result<Self> {
        let url = SensitiveUrl::parse(beacon_url).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to parse beacon URL '{}': {:?}", beacon_url, e)
        })?;

        // Configure timeouts - adjust as needed
        let timeouts = Timeouts::set_all(Duration::from_secs(30));

        let endpoint = url.full.clone();
        let client = BeaconNodeHttpClient::new(url, timeouts);

        Ok(Self { client, endpoint })
    }

    /// Performs a GET request to a beacon API path and deserializes the JSON response
    async fn get_json<T: DeserializeOwned>(
        &self,
        path: &[&str],
        query: &[(&str, &str)],
    ) -> Result<T> {
        let mut url = self.endpoint.clone();
        url.path_segments_mut()
            .map_err(|()| color_eyre::eyre::eyre!("Invalid beacon URL: {}", self.endpoint))?
            .pop_if_empty()
            .extend(path);
        url.query_pairs_mut().extend_pairs(query);

        let response = self
            .client
            .get_response(url.clone(), |builder| builder.accept(Accept::Json))
            .await
            .map_err(|e| {
                color_eyre::eyre::eyre!("Beacon API request to {} failed: {:?}", url, e)
            })?;

        response
            .json()
            .await
            .wrap_err_with(|| format!("Failed to deserialize beacon API response from {}", url))
    }

    /// Fetch light client update for a specific period
    pub async fn fetch_period_update(&self, period: u64) -> Result<BorshLightClientUpdate> {
        let updates: Vec<VersionedResponse<BorshLightClientUpdate>> = self
            .get_json(
                &["eth", "v1", "beacon", "light_client", "updates"],
                &[("start_period", &period.to_string()), ("count", "1")],
            )
            .await
            .wrap_err_with(|| {
                format!("Failed to fetch light client update for period {}", period)
            })?;

        let update = updates.into_iter().next().ok_or_else(|| {
            color_eyre::eyre::eyre!("No light client update found for period {}", period)
        })?;

        Ok(update.data)
    }

    /// Fetch latest finality update
    pub async fn fetch_finality_update(&self) -> Result<BorshLightClientUpdate> {
        let finality_update: VersionedResponse<BorshLightClientUpdate> = self
            .get_json(
                &["eth", "v1", "beacon", "light_client", "finality_update"],
                &[],
            )
            .await
            .wrap_err("Failed to fetch light client finality update")?;

        Ok(finality_update.data)
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

    /// Calculate sync committee period for a given slot
    pub fn get_period_for_slot(slot: u64) -> u64 {
        slot / (SLOTS_PER_EPOCH * EPOCHS_PER_PERIOD)
    }
}
