use serde_json::Value;
use std::error::Error;
use types::BeaconBlockBody;
use types::BeaconBlockHeader;
use types::MainnetEthSpec;

async fn get_json_from_raw_request(url: &str) -> Result<std::string::String, reqwest::Error> {
    reqwest::get(url).await?.text().await
}

/// `BeaconRPCClient` allows getting beacon block body and beacon block header
/// using Beacon RPC API (https://ethereum.github.io/beacon-APIs/)
pub struct BeaconRPCClient {
    endpoint_url: std::string::String,
}

impl BeaconRPCClient {
    const URL_HEADER_PATH: &'static str = "eth/v1/beacon/headers";
    const URL_BODY_PATH: &'static str = "eth/v2/beacon/blocks";

    /// Creates `BeaconRPCClient` for the given BeaconAPI `endpoint_url`
    pub fn new(endpoint_url: &str) -> Self {
        Self {
            endpoint_url: endpoint_url.to_string(),
        }
    }

    /// Returns `BeaconBlockBody` struct for the given `block_id`.
    ///
    /// # Arguments
    ///
    /// * `block_id` - Block identifier. Can be one of: "head" (canonical head in node's view),
    /// "genesis", "finalized", <slot>, <hex encoded blockRoot with 0x prefix>
    /// (see https://ethereum.github.io/beacon-APIs/#/Beacon/getBlockV2)
    pub async fn get_beacon_block_body_for_block_id(
        &self,
        block_id: &str,
    ) -> Result<BeaconBlockBody<MainnetEthSpec>, Box<dyn Error>> {
        let url = format!("{}/{}/{}", self.endpoint_url, Self::URL_BODY_PATH, block_id);
        let body_json =
            &Self::get_body_json_from_rpc_result(&get_json_from_raw_request(&url).await?)?;
        Ok(serde_json::from_str(body_json)?)
    }

    /// Returns `BeaconBlockHeader` struct for the given `block_id`.
    ///
    /// # Arguments
    ///
    /// * `block_id` - Block identifier. Can be one of: "head" (canonical head in node's view),
    /// "genesis", "finalized", <slot>, <hex encoded blockRoot with 0x prefix>
    /// (see https://ethereum.github.io/beacon-APIs/#/Beacon/getBlockHeader)
    pub async fn get_beacon_block_header_for_block_id(
        &self,
        block_id: &str,
    ) -> Result<BeaconBlockHeader, Box<dyn Error>> {
        let url = format!(
            "{}/{}/{}",
            self.endpoint_url,
            Self::URL_HEADER_PATH,
            block_id
        );
        let json_str =
            Self::get_header_json_from_rpc_result(&get_json_from_raw_request(&url).await?)?;
        Ok(serde_json::from_str(&json_str)?)
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
}

impl Default for BeaconRPCClient {
    fn default() -> Self {
        Self::new("https://lodestar-kiln.chainsafe.io")
    }
}

#[cfg(test)]
mod tests {
    use crate::beacon_rpc_client::BeaconRPCClient;
    use crate::test_utils::read_json_file_from_data_dir;
    use types::BeaconBlockBody;
    use types::BeaconBlockHeader;
    use types::MainnetEthSpec;

    const TEST_BEACON_BLOCK_ID: u32 = 741888;

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

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_get_json_from_raw_request() {
        let file_json_str = read_json_file_from_data_dir("beacon_block_kiln_slot_741888.json");

        let url = "https://lodestar-kiln.chainsafe.io/eth/v2/beacon/blocks/741888";
        let rpc_json_str = aw!(crate::beacon_rpc_client::get_json_from_raw_request(url));

        assert_eq!(rpc_json_str.unwrap(), file_json_str.trim());
    }

    #[test]
    fn test_rpc_beacon_block_body_and_header_smoke() {
        let _beacon_block_body = aw!(BeaconRPCClient::default()
            .get_beacon_block_body_for_block_id(&TEST_BEACON_BLOCK_ID.to_string()))
        .unwrap();
        let _beacon_block_header = aw!(BeaconRPCClient::default()
            .get_beacon_block_header_for_block_id(&TEST_BEACON_BLOCK_ID.to_string()))
        .unwrap();
    }

    #[test]
    fn test_get_beacon_block_header() {
        let beacon_block_header = aw!(BeaconRPCClient::default()
            .get_beacon_block_header_for_block_id(&TEST_BEACON_BLOCK_ID.to_string()))
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
        let beacon_block_body = aw!(BeaconRPCClient::default()
            .get_beacon_block_body_for_block_id(&TEST_BEACON_BLOCK_ID.to_string()))
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
}
