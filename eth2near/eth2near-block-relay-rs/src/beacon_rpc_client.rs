use crate::execution_block_proof::ExecutionBlockProof;
use crate::relay_errors::{
    ExecutionPayloadError, MissSyncAggregationError, SignatureSlotNotFoundError,
};
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
}

impl BeaconRPCClient {
    const URL_HEADER_PATH: &'static str = "eth/v1/beacon/headers";
    const URL_BODY_PATH: &'static str = "eth/v2/beacon/blocks";
    const URL_GET_LIGHT_CLIENT_UPDATE_API: &'static str = "eth/v1/light_client/updates";
    const URL_FINALITY_LIGHT_CLIENT_UPDATE_PATH: &'static str =
        "eth/v1/light_client/finality_update/";
    const URL_STATE_PATH: &'static str = "eth/v2/debug/beacon/states";

    const SLOTS_PER_EPOCH: u64 = 32;
    const EPOCHS_PER_PERIOD: u64 = 256;

    /// Creates `BeaconRPCClient` for the given BeaconAPI `endpoint_url`
    pub fn new(endpoint_url: &str) -> Self {
        Self {
            endpoint_url: endpoint_url.to_string(),
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(180))
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
        let body_json =
            &Self::get_body_json_from_rpc_result(&self.get_json_from_raw_request(&url)?)?;
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

        let json_str =
            Self::get_header_json_from_rpc_result(&self.get_json_from_raw_request(&url)?)?;
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
        let beacon_block_hash_str: String = serde_json::to_string(&beacon_block_hash)?;
        let beacon_block_hash_str = &beacon_block_hash_str[1..beacon_block_hash_str.len() - 1];

        let url = format!(
            "{}/{}/{}",
            self.endpoint_url,
            Self::URL_BODY_PATH,
            beacon_block_hash_str
        );
        let block_json_str = &self.get_json_from_raw_request(&url)?;
        let v: Value = serde_json::from_str(block_json_str)?;
        let slot = v["data"]["message"]["slot"].to_string();
        let slot = slot[1..slot.len() - 1].parse::<u64>()?;

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
        let json_str = self.get_json_from_raw_request(&url_request)?;

        let v: Value = serde_json::from_str(&json_str)?;
        let state_json_str = serde_json::to_string(&v["data"])?;

        Ok(serde_json::from_str(&state_json_str)?)
    }

    fn get_json_from_raw_request(&self, url: &str) -> Result<String, reqwest::Error> {
        trace!(target: "relay", "Beacon chain request: {}", url);
        self.client.get(url).send()?.text()
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
}

#[cfg(test)]
mod tests {
    use crate::beacon_block_header_with_execution_data::BeaconBlockHeaderWithExecutionData;
    use crate::beacon_rpc_client::BeaconRPCClient;
    use crate::test_utils::read_json_file_from_data_dir;
    use types::BeaconBlockBody;
    use types::BeaconBlockHeader;
    use types::MainnetEthSpec;

    const TEST_BEACON_BLOCK_ID: u32 = 741888;
    const BEACON_ENDPOINT: &str = "https://lodestar-kiln.chainsafe.io";

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
        let file_json_str = read_json_file_from_data_dir("beacon_block_kiln_slot_741888.json");

        let url = "https://lodestar-kiln.chainsafe.io/eth/v2/beacon/blocks/741888";
        let beacon_rpc_client = BeaconRPCClient::new(url);
        let rpc_json_str = beacon_rpc_client.get_json_from_raw_request(url);

        assert_eq!(rpc_json_str.unwrap(), file_json_str.trim());
    }

    #[test]
    fn test_rpc_beacon_block_body_and_header_smoke() {
        let _beacon_block_body = BeaconRPCClient::new(BEACON_ENDPOINT)
            .get_beacon_block_body_for_block_id(&TEST_BEACON_BLOCK_ID.to_string())
            .unwrap();
        let _beacon_block_header = BeaconRPCClient::new(BEACON_ENDPOINT)
            .get_beacon_block_header_for_block_id(&TEST_BEACON_BLOCK_ID.to_string())
            .unwrap();
    }

    #[test]
    fn test_get_beacon_block_header() {
        let beacon_block_header = BeaconRPCClient::new(BEACON_ENDPOINT)
            .get_beacon_block_header_for_block_id(&TEST_BEACON_BLOCK_ID.to_string())
            .unwrap();

        assert_eq!(beacon_block_header.slot, 741888);
        assert_eq!(beacon_block_header.proposer_index, 5407);
        assert_eq!(
            format!("{:?}", beacon_block_header.body_root),
            "0xd7f1c80baaceb9a1d3301e4f740fe8b5de9970153dc2ab254a4be39fe054addc"
        );
        assert_eq!(
            format!("{:?}", beacon_block_header.parent_root),
            "0xc94fa44bbe07890c887a50fff10a329da3c1ea7163ed08780f6d8c194b1e2904"
        );
        assert_eq!(
            format!("{:?}", beacon_block_header.state_root),
            "0x8725ba537c2f4449def8d06000b2136c40bca189cf24380f334b5edb41635507"
        );
    }

    #[test]
    fn test_get_beacon_block_body() {
        let beacon_block_body = BeaconRPCClient::new(BEACON_ENDPOINT)
            .get_beacon_block_body_for_block_id(&TEST_BEACON_BLOCK_ID.to_string())
            .unwrap();
        assert_eq!(beacon_block_body.attestations().len(), 29);
        assert_eq!(
            format!("{:?}", beacon_block_body.eth1_data().block_hash),
            "0x95a8bfef2aa4b30e63647f0e8eef7352ebac10a066acf8e24c3387982faffae2"
        );
        assert_eq!(beacon_block_body.eth1_data().deposit_count, 16392);
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
        const PERIOD: u64 = 100;
        let beacon_rpc_client = BeaconRPCClient::new(BEACON_ENDPOINT);
        let light_client_update = beacon_rpc_client.get_light_client_update(PERIOD).unwrap();

        // check attested_header
        assert_eq!(light_client_update.attested_beacon_header.slot, 823724);
        assert_eq!(
            light_client_update.attested_beacon_header.proposer_index,
            105744
        );
        assert_eq!(
            serde_json::to_string(&light_client_update.attested_beacon_header.parent_root).unwrap(),
            "\"0xb059eecb214d18b7ee03e73a4094bce92e2e302b91f542a0412508fd5fd7b4fe\""
        );
        assert_eq!(
            serde_json::to_string(&light_client_update.attested_beacon_header.state_root).unwrap(),
            "\"0x2cb304118032eb9c26dcfbf0217c3e4b4bf6bbd531193e9dea5f310c373bb482\""
        );
        assert_eq!(
            serde_json::to_string(&light_client_update.attested_beacon_header.body_root).unwrap(),
            "\"0x19bbc81a1aaf030cb5773c0370af542f9d6a2c5d13280427ddb8dbcca7dcdcb9\""
        );

        // check sync_aggregate
        assert_eq!(serde_json::to_string(&light_client_update.sync_aggregate.sync_committee_signature).unwrap(), "\"0x884ed0aeeac15090bd8ea39bcdb30e6586a53fd4d51237840e8f0c457942410b374cd2328c9d8c777b076fe79c0cc477048ad1ec7c4542f88f9e033752a2ea36ba2d39f6ef788381040e553e914fcf09d3cc5106d708c36478bc34e0370d1e41\"");

        // check signature_slot
        let beacon_block_body = beacon_rpc_client
            .get_beacon_block_body_for_block_id(&format!("{}", light_client_update.signature_slot))
            .unwrap();
        assert_eq!(serde_json::to_string(&beacon_block_body.sync_aggregate().unwrap().sync_committee_signature).unwrap(), "\"0x884ed0aeeac15090bd8ea39bcdb30e6586a53fd4d51237840e8f0c457942410b374cd2328c9d8c777b076fe79c0cc477048ad1ec7c4542f88f9e033752a2ea36ba2d39f6ef788381040e553e914fcf09d3cc5106d708c36478bc34e0370d1e41\"");

        // check finality_update
        let finality_update = light_client_update.finality_update;
        assert_eq!(finality_update.header_update.beacon_header.slot, 823648);
        assert_eq!(
            serde_json::to_string(&finality_update.header_update.beacon_header.body_root).unwrap(),
            "\"0x44c9d4b7b97a9e147cff85f90e68f8c30dae846fd6b969e6b8298e4d8311769e\""
        );
        assert_eq!(
            serde_json::to_string(&finality_update.finality_branch[1]).unwrap(),
            "\"0xf6c4677e9f110179f08f4eb6ad73e8a8b7d74a46f3956c66e4eb01ce6b70e5c4\""
        );

        // check sync_committe_update
        let sync_committe_update = light_client_update.sync_committee_update.unwrap();
        assert_eq!(serde_json::to_string(&sync_committe_update.next_sync_committee.aggregate_pubkey).unwrap(), "\"0x8dcf66eb3c34854131fd7926cd86c0d10051ad045e1fbc30c59e9162019b46f2daa0e0914bf1b32541dfa54b7be57c7c\"");
        assert_eq!(
            serde_json::to_string(&sync_committe_update.next_sync_committee_branch[1]).unwrap(),
            "\"0xedeb16e5754a4be920bb51e97dbf15833f838a5770e8509cc34cde12ee74422e\""
        );
    }

    // a utility function which prints JSON for last `LightClientUpdate`
    #[test]
    #[ignore]
    fn utility_show_get_light_client_update() {
        let light_client_update_fetcher = BeaconRPCClient::new(BEACON_ENDPOINT);
        let period = BeaconRPCClient::get_period_for_slot(
            light_client_update_fetcher
                .get_last_slot_number()
                .unwrap()
                .as_u64(),
        );

        let light_client_update = light_client_update_fetcher
            .get_light_client_update(period)
            .unwrap();
        let light_client_update_json_str = serde_json::to_string(&light_client_update).unwrap();

        println!(
            "Light client update pariod={}: {}",
            period, light_client_update_json_str
        );
    }

    // a utility function that prints JSON strings for all `BeaconBlockHeader`s with `ExecutionData` in specific range
    #[test]
    #[ignore]
    fn utility_show_headers_jsons_for_light_client_update() {
        let beacon_rpc_client = BeaconRPCClient::new(BEACON_ENDPOINT);
        let mut beacon_block_ext_headers: Vec<BeaconBlockHeaderWithExecutionData> = Vec::new();
        for slot in 823648..=827470 {
            let mut count = 1;
            loop {
                if let Ok(beacon_header) =
                    beacon_rpc_client.get_beacon_block_header_for_block_id(&format!("{}", slot))
                {
                    if let Ok(beacon_body) =
                        beacon_rpc_client.get_beacon_block_body_for_block_id(&format!("{}", slot))
                    {
                        beacon_block_ext_headers.push(
                            BeaconBlockHeaderWithExecutionData::new(beacon_header, &beacon_body)
                                .unwrap(),
                        );

                        println!(
                            "{},",
                            serde_json::to_string(
                                &beacon_block_ext_headers[beacon_block_ext_headers.len() - 1]
                            )
                            .unwrap()
                        );
                        break;
                    }
                }
                count += 1;
                if count > 3 {
                    break;
                }
            }
        }
    }
}
