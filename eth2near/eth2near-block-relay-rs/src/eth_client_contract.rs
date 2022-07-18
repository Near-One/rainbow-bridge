use std::fs;
use std::fs::File;
use eth_types::eth2::LightClientUpdate;
use crate::beacon_block_header_with_execution_data::BeaconBlockHeaderWithExecutionData;
use std::vec::Vec;
use std::string::String;
use std::path::Path;
use std::io::Write;
use near_crypto::InMemorySigner;
use near_jsonrpc_client::JsonRpcClient;
use near_sdk::AccountId;
use serde_json::Value;
use tokio::runtime::Handle;

pub struct EthClientContract {
    last_slot: u64,
    last_period: u64,
    dir_path: String,
    client: JsonRpcClient,
    contract_account: AccountId,
    signer: InMemorySigner,
}

impl EthClientContract {
    pub fn new(near_endpoint: &str, signer_account_id: &str,
               path_to_signer_secret_key: &str, contract_account_id: &str,
               last_slot: u64, dir_path: String) -> Self {
        fs::create_dir_all(&dir_path).unwrap();
        let last_period = last_slot/(32*256) - 1;

        let client = JsonRpcClient::connect(near_endpoint);
        let contract_account = contract_account_id.parse().unwrap();

        let signer_account_id = signer_account_id.parse().unwrap();
        let v: Value = serde_json::from_str(&std::fs::read_to_string(path_to_signer_secret_key).expect("Unable to read file")).unwrap();
        let signer_secret_key = serde_json::to_string(&v["private_key"]).unwrap();
        let signer_secret_key = &signer_secret_key[1..signer_secret_key.len() - 1];

        let signer = InMemorySigner::from_secret_key(signer_account_id, signer_secret_key.parse().unwrap());

        EthClientContract {
            last_slot: last_slot,
            last_period: last_period,
            dir_path,
            client,
            contract_account,
            signer,
        }
    }

    pub fn get_last_slot(&self) -> u64 {
        return self.last_slot;
    }

    pub fn get_last_period(&self) -> u64 {
        return self.last_period;
    }

    pub fn send_light_client_update(& mut self, light_client_update: LightClientUpdate, last_period: u64) {
        println!("Send light client update for period={}", last_period);

        let filename = format!("light_client_update_period_{}_attested_slot_{}.json", last_period, light_client_update.attested_header.slot);
        let light_client_update_out_path = Path::new(&self.dir_path).join(filename);
        let light_client_update_json_str = serde_json::to_string(&light_client_update).unwrap();

        let mut file = File::create(light_client_update_out_path).unwrap();
        file.write_all(light_client_update_json_str.as_bytes()).unwrap();

        self.last_period = last_period;
    }

    pub fn send_headers(& mut self, headers: Vec<BeaconBlockHeaderWithExecutionData>) {
        println!("Send headers, #headers = {} ", headers.len());

        if headers.len() == 0 {
            return;
        }

        let headers_filename = format!("headers_slots_{}_{}.json",
                                       headers[0].header.slot.as_u64(),
                                       headers[headers.len() - 1].header.slot.as_u64());
        let header_path = Path::new(&self.dir_path).join(headers_filename);
        let headers_json_str = serde_json::to_string(&headers).unwrap();

        let mut file = File::create(header_path).unwrap();
        file.write_all(headers_json_str.as_bytes()).unwrap();

        self.last_slot = headers[headers.len() - 1].header.slot.as_u64();
    }
}