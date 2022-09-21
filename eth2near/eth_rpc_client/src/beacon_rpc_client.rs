use crate::execution_block_proof::ExecutionBlockProof;
use crate::errors::{
    ExecutionPayloadError, FailOnGettingJson, MissSyncAggregationError, NoBlockForSlotError,
    SignatureSlotNotFoundError,
};
use crate::utils::trim_quotes;
use eth_types::eth2::BeaconBlockHeader;
use eth_types::eth2::FinalizedHeaderUpdate;
use eth_types::eth2::HeaderUpdate;
use eth_types::eth2::LightClientUpdate;
use eth_types::eth2::Slot;
use eth_types::eth2::SyncAggregate;
use eth_types::eth2::SyncCommittee;
use eth_types::eth2::SyncCommitteeUpdate;
use eth_types::H256;
use log::trace;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::error::Error;
use std::string::String;
use std::time::Duration;
use types::MainnetEthSpec;
use types::{BeaconBlockBody, BeaconState};

/// `BeaconRPCClient` allows getting beacon block body, beacon block header
/// and light client updates
/// using Beacon RPC API (https://ethereum.github.io/beacon-APIs/)
pub struct BeaconRPCClient {
    endpoint_url: String,
    client: Client,
    client_state_request: Client,
}

impl BeaconRPCClient {
    const URL_HEADER_PATH: &'static str = "eth/v1/beacon/headers";
    const URL_BODY_PATH: &'static str = "eth/v2/beacon/blocks";
    const URL_GET_LIGHT_CLIENT_UPDATE_API: &'static str = "eth/v1/beacon/light_client/updates";
    const URL_FINALITY_LIGHT_CLIENT_UPDATE_PATH: &'static str =
        "eth/v1/beacon/light_client/finality_update/";
    const URL_STATE_PATH: &'static str = "eth/v2/debug/beacon/states";

    const SLOTS_PER_EPOCH: u64 = 32;
    const EPOCHS_PER_PERIOD: u64 = 256;

    /// Creates `BeaconRPCClient` for the given BeaconAPI `endpoint_url`
    pub fn new(endpoint_url: &str, timeout_seconds: u64, timeout_state_seconds: u64) -> Self {
        Self {
            endpoint_url: endpoint_url.to_string(),
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(timeout_seconds))
                .build()
                .unwrap(),
            client_state_request: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(timeout_state_seconds))
                .build()
                .unwrap(),
        }
    }

    /// Returns `BeaconBlockBody` struct for the given `block_id`.
    ///
    /// # Arguments
    ///
    /// * `block_id` - Block identifier. Can be one of: "head" (canonical head in node's view),
    /// "genesis", "finalized", <slot>, <hex encoded blockRoot with 0x prefix>
    /// (see https://ethereum.github.io/beacon-APIs/#/Beacon/getBlockV2)
    pub fn get_beacon_block_body_for_block_id(
        &self,
        block_id: &str,
    ) -> Result<BeaconBlockBody<MainnetEthSpec>, Box<dyn Error>> {
        let url = format!("{}/{}/{}", self.endpoint_url, Self::URL_BODY_PATH, block_id);

        let json_str = &self.get_json_from_raw_request(&url)?;

        self.check_block_found_for_slot(json_str)?;
        let body_json = &Self::get_body_json_from_rpc_result(json_str)?;

        Ok(serde_json::from_str(body_json)?)
    }

    /// Returns `BeaconBlockHeader` struct for the given `block_id`.
    ///
    /// # Arguments
    ///
    /// * `block_id` - Block identifier. Can be one of: "head" (canonical head in node's view),
    /// "genesis", "finalized", <slot>, <hex encoded blockRoot with 0x prefix>
    /// (see https://ethereum.github.io/beacon-APIs/#/Beacon/getBlockHeader)
    pub fn get_beacon_block_header_for_block_id(
        &self,
        block_id: &str,
    ) -> Result<types::BeaconBlockHeader, Box<dyn Error>> {
        let url = format!(
            "{}/{}/{}",
            self.endpoint_url,
            Self::URL_HEADER_PATH,
            block_id
        );

        let json_str = &self.get_json_from_raw_request(&url)?;
        self.check_block_found_for_slot(json_str)?;
        let json_str = Self::get_header_json_from_rpc_result(json_str)?;
        Ok(serde_json::from_str(&json_str)?)
    }

    /// Returns `LightClientUpdate` struct for the given `period`.
    ///
    /// # Arguments
    ///
    /// * `period` - period id for which `LightClientUpdate` is fetched.
    /// On Mainnet, one period consists of 256 epochs, and one epoch consists of 32 slots
    pub fn get_light_client_update(
        &self,
        period: u64,
    ) -> Result<LightClientUpdate, Box<dyn Error>> {
        let url = format!(
            "{}/{}?start_period={}&count=1",
            self.endpoint_url,
            Self::URL_GET_LIGHT_CLIENT_UPDATE_API,
            period
        );
        let light_client_update_json_str = self.get_json_from_raw_request(&url)?;

        Ok(LightClientUpdate {
            attested_beacon_header: Self::get_attested_header_from_light_client_update_json_str(
                &light_client_update_json_str,
            )?,
            sync_aggregate: Self::get_sync_aggregate_from_light_client_update_json_str(
                &light_client_update_json_str,
            )?,
            signature_slot: self.get_signature_slot(&light_client_update_json_str)?,
            finality_update: self.get_finality_update_from_light_client_update_json_str(
                &light_client_update_json_str,
            )?,
            sync_committee_update: Some(
                Self::get_sync_committee_update_from_light_lient_update_json_str(
                    &light_client_update_json_str,
                )?,
            ),
        })
    }

    /// Return the last finalized slot in the Beacon chain
    pub fn get_last_finalized_slot_number(&self) -> Result<types::Slot, Box<dyn Error>> {
        Ok(self.get_beacon_block_header_for_block_id("finalized")?.slot)
    }

    /// Return the last slot in the Beacon chain
    pub fn get_last_slot_number(&self) -> Result<types::Slot, Box<dyn Error>> {
        Ok(self.get_beacon_block_header_for_block_id("head")?.slot)
    }

    pub fn get_slot_by_beacon_block_root(
        &self,
        beacon_block_hash: H256,
    ) -> Result<u64, Box<dyn Error>> {
        let beacon_block_hash_str: String = trim_quotes(serde_json::to_string(&beacon_block_hash)?);

        let url = format!(
            "{}/{}/{}",
            self.endpoint_url,
            Self::URL_BODY_PATH,
            beacon_block_hash_str
        );
        let block_json_str = &self.get_json_from_raw_request(&url)?;
        let v: Value = serde_json::from_str(block_json_str)?;
        let slot = trim_quotes(v["data"]["message"]["slot"].to_string()).parse::<u64>()?;

        Ok(slot)
    }

    pub fn get_block_number_for_slot(&self, slot: types::Slot) -> Result<u64, Box<dyn Error>> {
        let beacon_block_body = self.get_beacon_block_body_for_block_id(&slot.to_string())?;
        Ok(beacon_block_body
            .execution_payload()
            .map_err(|_| ExecutionPayloadError)?
            .execution_payload
            .block_number)
    }

    pub fn get_finality_light_client_update(&self) -> Result<LightClientUpdate, Box<dyn Error>> {
        let url = format!(
            "{}/{}",
            self.endpoint_url,
            Self::URL_FINALITY_LIGHT_CLIENT_UPDATE_PATH
        );

        let light_client_update_json_str = self.get_json_from_raw_request(&url)?;
        let v: Value = serde_json::from_str(&light_client_update_json_str)?;
        let light_client_update_json_str = serde_json::to_string(&json!({"data": [v["data"]]}))?;

        Ok(LightClientUpdate {
            attested_beacon_header: Self::get_attested_header_from_light_client_update_json_str(
                &light_client_update_json_str,
            )?,
            sync_aggregate: Self::get_sync_aggregate_from_light_client_update_json_str(
                &light_client_update_json_str,
            )?,
            signature_slot: self.get_signature_slot(&light_client_update_json_str)?,
            finality_update: self.get_finality_update_from_light_client_update_json_str(
                &light_client_update_json_str,
            )?,
            sync_committee_update: None::<SyncCommitteeUpdate>,
        })
    }

    pub fn get_finality_light_client_update_with_sync_commity_update(
        &self,
    ) -> Result<LightClientUpdate, Box<dyn Error>> {
        let url_finality = format!(
            "{}/{}",
            self.endpoint_url,
            Self::URL_FINALITY_LIGHT_CLIENT_UPDATE_PATH
        );
        let last_period = Self::get_period_for_slot(self.get_last_slot_number()?.as_u64());
        let url_update = format!(
            "{}/{}?start_period={}&count=1",
            self.endpoint_url,
            Self::URL_GET_LIGHT_CLIENT_UPDATE_API,
            last_period
        );
        let finality_light_client_update_json_str =
            self.get_json_from_raw_request(&url_finality)?;
        let light_client_update_json_str = self.get_json_from_raw_request(&url_update)?;

        let v: Value = serde_json::from_str(&finality_light_client_update_json_str)?;
        let finality_light_client_update_json_str =
            serde_json::to_string(&json!({"data": [v["data"]]}))?;

        Ok(LightClientUpdate {
            attested_beacon_header: Self::get_attested_header_from_light_client_update_json_str(
                &finality_light_client_update_json_str,
            )?,
            sync_aggregate: Self::get_sync_aggregate_from_light_client_update_json_str(
                &finality_light_client_update_json_str,
            )?,
            signature_slot: self.get_signature_slot(&finality_light_client_update_json_str)?,
            finality_update: self.get_finality_update_from_light_client_update_json_str(
                &finality_light_client_update_json_str,
            )?,
            sync_committee_update: Some(
                Self::get_sync_committee_update_from_light_lient_update_json_str(
                    &light_client_update_json_str,
                )?,
            ),
        })
    }

    pub fn get_beacon_state(
        &self,
        state_id: &str,
    ) -> Result<BeaconState<MainnetEthSpec>, Box<dyn Error>> {
        let url_request = format!(
            "{}/{}/{}",
            self.endpoint_url,
            Self::URL_STATE_PATH,
            state_id
        );
        let json_str = Self::get_json_from_client(&self.client_state_request, &url_request)?;

        let v: Value = serde_json::from_str(&json_str)?;
        let state_json_str = serde_json::to_string(&v["data"])?;

        Ok(serde_json::from_str(&state_json_str)?)
    }

    pub fn is_syncing(&self) -> Result<bool, Box<dyn Error>> {
        let url_request = format!("{}/eth/v1/node/syncing", self.endpoint_url);
        let json_str = self.get_json_from_raw_request(&url_request)?;

        let v: Value = serde_json::from_str(&json_str)?;
        Ok(v["data"]["is_syncing"].as_bool().unwrap())
    }

    fn get_json_from_client(client: &Client, url: &str) -> Result<String, Box<dyn Error>> {
        trace!(target: "relay", "Beacon chain request: {}", url);
        let json_str = client.get(url).send()?.text()?;
        if let Err(_) = serde_json::from_str::<Value>(&json_str) {
            return Err(Box::new(FailOnGettingJson { response: json_str }));
        }

        Ok(json_str)
    }

    fn get_json_from_raw_request(&self, url: &str) -> Result<String, Box<dyn Error>> {
        Self::get_json_from_client(&self.client, url)
    }

    fn get_body_json_from_rpc_result(
        block_json_str: &str,
    ) -> Result<std::string::String, Box<dyn Error>> {
        let v: Value = serde_json::from_str(block_json_str)?;
        let body_json_str = serde_json::to_string(&v["data"]["message"]["body"])?;
        Ok(body_json_str)
    }

    fn get_header_json_from_rpc_result(
        json_str: &str,
    ) -> Result<std::string::String, Box<dyn Error>> {
        let v: Value = serde_json::from_str(json_str)?;
        let hjson_str = serde_json::to_string(&v["data"]["header"]["message"])?;
        Ok(hjson_str)
    }

    fn get_attested_header_from_light_client_update_json_str(
        light_client_update_json_str: &str,
    ) -> Result<BeaconBlockHeader, Box<dyn Error>> {
        let v: Value = serde_json::from_str(light_client_update_json_str)?;
        let attested_header_json_str = serde_json::to_string(&v["data"][0]["attested_header"])?;
        let attested_header: BeaconBlockHeader = serde_json::from_str(&attested_header_json_str)?;

        Ok(attested_header)
    }

    fn get_sync_aggregate_from_light_client_update_json_str(
        light_client_update_json_str: &str,
    ) -> Result<SyncAggregate, Box<dyn Error>> {
        let v: Value = serde_json::from_str(light_client_update_json_str)?;
        let sync_aggregate_json_str = serde_json::to_string(&v["data"][0]["sync_aggregate"])?;
        let sync_aggregate: SyncAggregate = serde_json::from_str(&sync_aggregate_json_str)?;

        Ok(sync_aggregate)
    }

    // `signature_slot` is not provided in the current API. The slot is brute-forced
    // until `SyncAggregate` in `BeconBlockBody` in the current slot is equal
    // to `SyncAggregate` in `LightClientUpdate`
    fn get_signature_slot(
        &self,
        light_client_update_json_str: &str,
    ) -> Result<Slot, Box<dyn Error>> {
        const CHECK_SLOTS_FORWARD_LIMIT: u64 = 10;

        let v: Value = serde_json::from_str(light_client_update_json_str)?;

        let attested_header_json_str = serde_json::to_string(&v["data"][0]["attested_header"])?;
        let attested_header: BeaconBlockHeader = serde_json::from_str(&attested_header_json_str)?;

        let mut signature_slot = attested_header.slot + 1;

        let sync_aggregate = Self::get_sync_aggregate_from_light_client_update_json_str(
            light_client_update_json_str,
        )?;

        loop {
            if let Ok(beacon_block_body) =
                self.get_beacon_block_body_for_block_id(&format!("{}", signature_slot))
            {
                if format!(
                    "\"{:?}\"",
                    beacon_block_body
                        .sync_aggregate()
                        .map_err(|_| { MissSyncAggregationError })?
                        .sync_committee_signature
                ) == serde_json::to_string(&sync_aggregate.sync_committee_signature)?
                {
                    break;
                }
            }

            signature_slot += 1;
            if signature_slot - attested_header.slot > CHECK_SLOTS_FORWARD_LIMIT {
                return Err(Box::new(SignatureSlotNotFoundError));
            }
        }

        Ok(signature_slot)
    }

    fn get_finality_update_from_light_client_update_json_str(
        &self,
        light_client_update_json_str: &str,
    ) -> Result<FinalizedHeaderUpdate, Box<dyn Error>> {
        let v: Value = serde_json::from_str(light_client_update_json_str)?;

        let finalized_header_json_str = serde_json::to_string(&v["data"][0]["finalized_header"])?;
        let finalized_header: BeaconBlockHeader = serde_json::from_str(&finalized_header_json_str)?;

        let finalized_branch_json_str = serde_json::to_string(&v["data"][0]["finality_branch"])?;
        let finalized_branch: Vec<eth_types::H256> =
            serde_json::from_str(&finalized_branch_json_str)?;

        let finalized_block_slot = finalized_header.slot;

        let finalized_block_body =
            self.get_beacon_block_body_for_block_id(&format!("{}", finalized_block_slot))?;
        let finalized_block_eth1data_proof =
            ExecutionBlockProof::construct_from_beacon_block_body(&finalized_block_body)?;

        Ok(FinalizedHeaderUpdate {
            header_update: HeaderUpdate {
                beacon_header: finalized_header,
                execution_block_hash: eth_types::H256::from(
                    finalized_block_eth1data_proof
                        .get_execution_block_hash()
                        .0
                        .to_vec(),
                ),
                execution_hash_branch: finalized_block_eth1data_proof
                    .get_proof()
                    .iter()
                    .copied()
                    .map(|x| eth_types::H256::from(x.0.to_vec()))
                    .collect(),
            },
            finality_branch: finalized_branch,
        })
    }

    fn get_sync_committee_update_from_light_lient_update_json_str(
        light_client_update_json_str: &str,
    ) -> Result<SyncCommitteeUpdate, Box<dyn Error>> {
        let v: Value = serde_json::from_str(light_client_update_json_str)?;
        let next_sync_committee_branch_json_str =
            serde_json::to_string(&v["data"][0]["next_sync_committee_branch"])?;
        let next_sync_committee_branch: Vec<eth_types::H256> =
            serde_json::from_str(&next_sync_committee_branch_json_str)?;

        let next_sync_committee_json_str =
            serde_json::to_string(&v["data"][0]["next_sync_committee"])?;
        let next_sync_committee: SyncCommittee =
            serde_json::from_str(&next_sync_committee_json_str)?;

        Ok(SyncCommitteeUpdate {
            next_sync_committee,
            next_sync_committee_branch,
        })
    }

    pub fn get_period_for_slot(slot: u64) -> u64 {
        slot / (Self::SLOTS_PER_EPOCH * Self::EPOCHS_PER_PERIOD)
    }

    pub fn get_non_empty_beacon_block_header(
        &self,
        start_slot: u64,
    ) -> Result<types::BeaconBlockHeader, Box<dyn Error>> {
        const CHECK_SLOTS_FORWARD_LIMIT: u64 = 32;

        let mut slot = start_slot;
        for _ in 0..CHECK_SLOTS_FORWARD_LIMIT {
            if let Ok(beacon_block_body) =
                self.get_beacon_block_header_for_block_id(&format!("{}", slot))
            {
                return Ok(beacon_block_body);
            }
            slot += 1;
        }

        Err(format!(
            "Unable to get non empty beacon block in range [`{}`-`{}`)",
            start_slot,
            start_slot + CHECK_SLOTS_FORWARD_LIMIT
        ))?
    }

    fn check_block_found_for_slot(&self, json_str: &str) -> Result<(), Box<dyn Error>> {
        let parse_json: Value = serde_json::from_str(json_str)?;
        if parse_json.is_object() {
            if let Some(msg_str) = parse_json["message"].as_str() {
                if msg_str.contains("No block found for") {
                    return Err(Box::new(NoBlockForSlotError));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::beacon_rpc_client::BeaconRPCClient;
    use crate::config_for_tests::ConfigForTests;
    use crate::utils::read_json_file_from_data_dir;
    use serde_json::Value;
    use types::BeaconBlockBody;
    use types::BeaconBlockHeader;
    use types::MainnetEthSpec;
    use crate::utils::trim_quotes;

    const TIMEOUT_SECONDS: u64 = 30;
    const TIMEOUT_STATE_SECONDS: u64 = 1000;

    fn get_config() -> ConfigForTests {
        ConfigForTests::load_from_toml("config_for_tests.toml".try_into().unwrap())
    }

    #[test]
    fn test_get_header_from_json() {
        let beacon_block_header_json_str = r#"
        {
            "slot": "0",
            "proposer_index": "1",
            "parent_root": "0x1cfedbc04788917c188bdab08bf1ed4ece4f352782b61989e142a211fe876c4c",
            "state_root": "0xc40e5fae29997182dbafa0e091d41b27d9bbd6ac388df271e9224d3c0240017f",
            "body_root": "0xb4d27c714e935a2970033c00ebb1d756336ded865e84fd22bec3395971158ab6"
        }
        "#;

        let beacon_block_header: BeaconBlockHeader =
            serde_json::from_str(beacon_block_header_json_str).unwrap();

        assert_eq!(beacon_block_header.slot, 0);
        assert_eq!(beacon_block_header.proposer_index, 1);
        assert_eq!(
            format!("{:?}", beacon_block_header.body_root),
            "0xb4d27c714e935a2970033c00ebb1d756336ded865e84fd22bec3395971158ab6"
        );
        assert_eq!(
            format!("{:?}", beacon_block_header.parent_root),
            "0x1cfedbc04788917c188bdab08bf1ed4ece4f352782b61989e142a211fe876c4c"
        );
        assert_eq!(
            format!("{:?}", beacon_block_header.state_root),
            "0xc40e5fae29997182dbafa0e091d41b27d9bbd6ac388df271e9224d3c0240017f"
        );
    }

    #[test]
    fn test_get_beacon_body_from_json() {
        let beacon_block_body_json_str =
            read_json_file_from_data_dir("beacon_block_body_kiln_slot_741888.json");
        let beacon_block_body: BeaconBlockBody<MainnetEthSpec> =
            serde_json::from_str(&beacon_block_body_json_str).unwrap();

        assert_eq!(
            format!("{:?}", beacon_block_body.eth1_data().deposit_root),
            "0x4b2bfc129d2ce9b4264882bb49c5df18faa8d10b571ee7e87aa85e164da0d2d7"
        );
    }

    #[test]
    fn test_get_json_from_raw_request() {
        let config = get_config();
        let file_json_str =
            std::fs::read_to_string(&config.path_to_block).expect("Unable to read file");

        let url = format!(
            "{}/eth/v2/beacon/blocks/{}",
            config.beacon_endpoint, config.first_slot
        );
        let beacon_rpc_client = BeaconRPCClient::new(&url, TIMEOUT_SECONDS, TIMEOUT_STATE_SECONDS);
        let rpc_json_str = beacon_rpc_client.get_json_from_raw_request(&url);
        assert_eq!(rpc_json_str.unwrap(), file_json_str.trim());
    }

    #[test]
    fn test_rpc_beacon_block_body_and_header_smoke() {
        let config = get_config();

        let _beacon_block_body = BeaconRPCClient::new(
            &config.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        )
        .get_beacon_block_body_for_block_id(&config.first_slot.to_string())
        .unwrap();
        let _beacon_block_header = BeaconRPCClient::new(
            &config.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        )
        .get_beacon_block_header_for_block_id(&config.first_slot.to_string())
        .unwrap();
    }

    #[test]
    fn test_get_beacon_block_header() {
        let config = get_config();
        let beacon_block_header = BeaconRPCClient::new(
            &config.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        )
        .get_beacon_block_header_for_block_id(&format!("{}", config.first_slot))
        .unwrap();

        let header_json_str =
            std::fs::read_to_string(config.path_to_header).expect("Unable to read file");
        let v: Value = serde_json::from_str(&header_json_str).unwrap();

        assert_eq!(
            beacon_block_header.slot,
            trim_quotes(v["data"]["header"]["message"]["slot"].to_string())
                .parse::<u64>()
                .unwrap()
        );
        assert_eq!(
            beacon_block_header.proposer_index,
            trim_quotes(v["data"]["header"]["message"]["proposer_index"].to_string())
                .parse::<u64>()
                .unwrap()
        );
        assert_eq!(
            format!("{:?}", beacon_block_header.body_root),
            trim_quotes(v["data"]["header"]["message"]["body_root"].to_string())
        );
        assert_eq!(
            format!("{:?}", beacon_block_header.parent_root),
            trim_quotes(v["data"]["header"]["message"]["parent_root"].to_string())
        );
        assert_eq!(
            format!("{:?}", beacon_block_header.state_root),
            trim_quotes(v["data"]["header"]["message"]["state_root"].to_string())
        );
    }

    #[test]
    fn test_get_beacon_block_body() {
        let config = get_config();

        let beacon_block_body = BeaconRPCClient::new(
            &config.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        )
        .get_beacon_block_body_for_block_id(&config.first_slot.to_string())
        .unwrap();

        let block_json_str =
            std::fs::read_to_string(config.path_to_block).expect("Unable to read file");
        let v: Value = serde_json::from_str(&block_json_str).unwrap();
        assert_eq!(
            beacon_block_body.attestations().len(),
            v["data"]["message"]["body"]["attestations"]
                .as_array()
                .unwrap()
                .len()
        );
        assert_eq!(
            format!("{:?}", beacon_block_body.eth1_data().block_hash),
            v["data"]["message"]["body"]["eth1_data"]["block_hash"]
        );
    }

    #[test]
    fn test_is_sync() {
        assert!(!BeaconRPCClient::new(
            "https://lodestar-goerli.chainsafe.io",
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS
        )
        .is_syncing()
        .unwrap());
    }

    #[test]
    fn test_get_header_json_from_rpc_result() {
        let beacon_block_header_response_json =
            read_json_file_from_data_dir("beacon_block_header_response_kiln_slot_741888.json");
        let beacon_block_header_struct_json =
            read_json_file_from_data_dir("beacon_block_header_struct_kiln_slot_741888.json");

        let beacon_header_file: BeaconBlockHeader =
            serde_json::from_str(&beacon_block_header_struct_json).unwrap();
        let beacon_header_rpc: BeaconBlockHeader = serde_json::from_str(
            &BeaconRPCClient::get_header_json_from_rpc_result(&beacon_block_header_response_json)
                .unwrap(),
        )
        .unwrap();

        assert_eq!(beacon_header_file, beacon_header_rpc);
    }

    #[test]
    fn test_beacon_block_body_json_from_rpc_result() {
        let beacon_block_json = read_json_file_from_data_dir("beacon_block_kiln_slot_741888.json");
        let beacon_block_body_json =
            read_json_file_from_data_dir("beacon_block_body_kiln_slot_741888.json");
        let beacon_body_file: BeaconBlockBody<MainnetEthSpec> =
            serde_json::from_str(&beacon_block_body_json).unwrap();
        let beacon_body_rpc: BeaconBlockBody<MainnetEthSpec> = serde_json::from_str(
            &BeaconRPCClient::get_body_json_from_rpc_result(&beacon_block_json).unwrap(),
        )
        .unwrap();

        assert_eq!(beacon_body_file, beacon_body_rpc);
    }

    #[test]
    fn test_fetch_light_client_update() {
        let config = get_config();

        let beacon_rpc_client = BeaconRPCClient::new(
            &config.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let file_json_str = std::fs::read_to_string(&config.path_to_light_client_update)
            .expect("Unable to read file");
        let v: Value = serde_json::from_str(&file_json_str).unwrap();

        let period: u64 = BeaconRPCClient::get_period_for_slot(
            v["data"][0]["attested_header"]["slot"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap(),
        );
        let light_client_update = beacon_rpc_client.get_light_client_update(period).unwrap();

        // check attested_header
        assert_eq!(
            light_client_update.attested_beacon_header.slot,
            v["data"][0]["attested_header"]["slot"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );
        assert_eq!(
            light_client_update.attested_beacon_header.proposer_index,
            v["data"][0]["attested_header"]["proposer_index"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );
        assert_eq!(
            serde_json::to_string(&light_client_update.attested_beacon_header.parent_root).unwrap(),
            format!(
                "\"{}\"",
                v["data"][0]["attested_header"]["parent_root"]
                    .as_str()
                    .unwrap()
            )
        );
        assert_eq!(
            serde_json::to_string(&light_client_update.attested_beacon_header.state_root).unwrap(),
            format!(
                "\"{}\"",
                v["data"][0]["attested_header"]["state_root"]
                    .as_str()
                    .unwrap()
            )
        );
        assert_eq!(
            serde_json::to_string(&light_client_update.attested_beacon_header.body_root).unwrap(),
            format!(
                "\"{}\"",
                v["data"][0]["attested_header"]["body_root"]
                    .as_str()
                    .unwrap()
            )
        );

        // check sync_aggregate
        assert_eq!(
            serde_json::to_string(&light_client_update.sync_aggregate.sync_committee_signature)
                .unwrap(),
            format!(
                "{}",
                v["data"][0]["sync_aggregate"]["sync_committee_signature"]
            )
        );

        // check signature_slot
        let beacon_block_body = beacon_rpc_client
            .get_beacon_block_body_for_block_id(&format!("{}", light_client_update.signature_slot))
            .unwrap();
        assert_eq!(
            serde_json::to_string(
                &beacon_block_body
                    .sync_aggregate()
                    .unwrap()
                    .sync_committee_signature
            )
            .unwrap(),
            format!(
                "{}",
                v["data"][0]["sync_aggregate"]["sync_committee_signature"]
            )
        );

        // check finality_update
        let finality_update = light_client_update.finality_update;
        assert_eq!(
            finality_update.header_update.beacon_header.slot,
            v["data"][0]["finalized_header"]["slot"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        );
        assert_eq!(
            serde_json::to_string(&finality_update.header_update.beacon_header.body_root).unwrap(),
            format!("{}", v["data"][0]["finalized_header"]["body_root"])
        );
        assert_eq!(
            serde_json::to_string(&finality_update.finality_branch[1]).unwrap(),
            format!("{}", v["data"][0]["finality_branch"][1])
        );

        // check sync_committe_update
        let sync_committe_update = light_client_update.sync_committee_update.unwrap();
        assert_eq!(
            serde_json::to_string(&sync_committe_update.next_sync_committee.aggregate_pubkey)
                .unwrap(),
            format!(
                "{}",
                v["data"][0]["next_sync_committee"]["aggregate_pubkey"]
            )
        );
        assert_eq!(
            serde_json::to_string(&sync_committe_update.next_sync_committee_branch[1]).unwrap(),
            format!("{}", v["data"][0]["next_sync_committee_branch"][1])
        );
    }
}
