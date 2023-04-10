use eth_types::BlockHeader;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::error::Error;

pub struct Eth1RPCClient {
    endpoint_url: String,
    client: Client,
}

impl Eth1RPCClient {
    pub fn new(endpoint_url: &str) -> Self {
        Self {
            endpoint_url: endpoint_url.to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn get_block_header_by_number(&self, number: u64) -> Result<BlockHeader, Box<dyn Error>> {
        let hex_str_number = format!("0x{:x}", number);
        let json_value = json!({
            "id": 0,
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": [hex_str_number, false]
        });

        let res = self
            .client
            .post(&self.endpoint_url)
            .json(&json_value)
            .send()?
            .text()?;

        let val: Value = serde_json::from_str(&res)?;
        let mut block_json = serde_json::to_string(&val["result"])?;

        block_json = block_json.replace("baseFeePerGas", "base_fee_per_gas");
        block_json = block_json.replace("extraData", "extra_data");
        block_json = block_json.replace("gasLimit", "gas_limit");
        block_json = block_json.replace("gasUsed", "gas_used");
        block_json = block_json.replace("logsBloom", "log_bloom");
        block_json = block_json.replace("mixHash", "mix_hash");
        block_json = block_json.replace("parentHash", "parent_hash");
        block_json = block_json.replace("receiptsRoot", "receipts_root");
        block_json = block_json.replace("sha3Uncles", "uncles_hash");
        block_json = block_json.replace("stateRoot", "state_root");
        block_json = block_json.replace("totalDifficulty", "total_difficulty");
        block_json = block_json.replace("transactionsRoot", "transactions_root");
        block_json = block_json.replace("parentHash", "parent_hash");
        block_json = block_json.replace("miner", "author");
        block_json = block_json.replace("withdrawalsRoot", "withdrawals_root");

        let block_header: BlockHeader = serde_json::from_str(&block_json)?;
        Ok(block_header)
    }

    pub fn is_syncing(&self) -> Result<bool, Box<dyn Error>> {
        let json_value = json!({
            "jsonrpc":"2.0",
            "method":"eth_syncing",
            "params":[],
            "id":1});

        let res = self
            .client
            .post(&self.endpoint_url)
            .json(&json_value)
            .send()?
            .text()?;

        let val: Value = serde_json::from_str(&res)?;
        let is_sync = val["result"].as_bool();
        if let Some(is_sync_val) = is_sync {
            return Ok(is_sync_val);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::config_for_tests::ConfigForTests;
    use crate::eth1_rpc_client::Eth1RPCClient;

    fn get_test_config() -> ConfigForTests {
        ConfigForTests::load_from_toml("config_for_tests.toml".try_into().unwrap())
    }

    #[test]
    fn test_smoke_get_block_by_number() {
        let config = get_test_config();

        let eth1_rpc_client = Eth1RPCClient::new(&config.eth1_endpoint);
        eth1_rpc_client
            .get_block_header_by_number(config.eth1_number)
            .unwrap();
    }

    #[test]
    fn test_is_syncing() {
        let config = get_test_config();

        let eth1_rpc_client = Eth1RPCClient::new(&config.eth1_endpoint);
        assert!(!eth1_rpc_client.is_syncing().unwrap());
    }
}
