use std::error::Error;
use eth_types::BlockHeader;
use reqwest::blocking::Client;
use serde_json::Value;

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

    pub fn get_block_header_by_number(
        &self,
        number: u64,
    ) -> Result<BlockHeader, Box<dyn Error>> {
        let json_str = format!("{}\"id\": 0,\
        \"jsonrpc\": \"2.0\",\
        \"method\": \"eth_getBlockByNumber\",\
        \"params\": [\"0x{:x}\",false]\
        {}", "{", number, "}");
        println!("json_str: {}", json_str);

        let value: Value = serde_json::from_str(&json_str).unwrap();
        println!("value: {:?}", value);

        let res = self.client.post(&self.endpoint_url).json(&value).send()?.text().unwrap();
        println!("res: {:?}", res);

        let val: Value = serde_json::from_str(&res).unwrap();
        let mut block_json = serde_json::to_string(&val["result"]).unwrap();

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

        println!("block_json: {}", block_json);
        let block_header: BlockHeader = serde_json::from_str(&block_json).unwrap();
        println!("block_heade: {:?}", block_header);
        Ok(block_header)
    }
}

impl Default for Eth1RPCClient {
    fn default() -> Self {
        Self::new("https://rpc.kiln.themerge.dev")
    }
}

#[cfg(test)]
mod tests {
    use crate::eth1_rpc_client::Eth1RPCClient;

    const TEST_BEACON_BLOCK_ID: u32 = 741888;

    #[test]
    fn test_smoke_get_block_by_number() {
        let eth1_rpc_client = Eth1RPCClient::default();
        eth1_rpc_client.get_block_header_by_number(766535);
    }
}
