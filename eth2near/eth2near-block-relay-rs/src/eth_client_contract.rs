use std::fs;
use std::fs::File;
use eth_types::eth2::LightClientUpdate;
use std::vec::Vec;
use std::string::String;
use std::path::Path;
use std::io::Write;
use borsh::BorshDeserialize;
use eth_types::{BlockHeader, H256};
use near_crypto::InMemorySigner;
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_client::methods;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::{AccountId, BlockReference, Finality, FunctionArgs, Nonce};
use serde_json::{json, Value};
use tokio::runtime::Runtime;
use near_primitives::borsh::BorshSerialize;
use near_primitives::views::QueryRequest;
use std::option::Option;

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

    pub fn get_last_submitted_slot(&self) -> u64 {
        return self.last_slot;
    }

    pub fn is_known_block(&self, execution_block_hash: &H256) -> bool {
        let result = self.call_view_function("is_known_execution_header".to_string(), execution_block_hash.try_to_vec().unwrap()).unwrap();
        let is_known: bool = bool::try_from_slice(&result).unwrap();
        is_known
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

        let rt = Runtime::new().unwrap();
        let handle = rt.handle();

        let access_key_query_response = handle.block_on(self.client
            .call(methods::query::RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id: self.signer.account_id.clone(),
                    public_key: self.signer.public_key.clone(),
                },
            })).unwrap();

        let current_nonce = self.get_current_nonce();
        let transaction = Transaction {
            signer_id: self.signer.account_id.clone(),
            public_key: self.signer.public_key.clone(),
            nonce: current_nonce + 1,
            receiver_id: self.contract_account.clone(),
            block_hash: access_key_query_response.block_hash,
            actions: vec![Action::FunctionCall(FunctionCallAction {
                method_name: "submit_update".to_string(),
                args: light_client_update.try_to_vec().unwrap(),
                gas: 100_000_000_000_000, // 100 TeraGas
                deposit: 0,
            })],
        };

        let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: transaction.sign(&self.signer),
        };

        handle.block_on(self.client.call(request)).unwrap();
    }

    pub fn get_finalized_beacon_block_hash(&self) -> H256 {
        let result = self.call_view_function("finalized_beacon_header_root".to_string(), json!({}).to_string().into_bytes()).unwrap();
        let beacon_block_hash: H256 = H256::try_from_slice(&result).unwrap();
        beacon_block_hash
    }

    fn call_view_function(&self, method_name: String, args: Vec<u8>) -> Option<Vec<u8>> {
        let rt = Runtime::new().unwrap();
        let handle = rt.handle();

        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id: self.contract_account.clone(),
                method_name,
                args: FunctionArgs::from(args),
            },
        };

        let response =  handle.block_on(self.client.call(request)).unwrap();
        println!("response: {:#?}", response);

        if let QueryResponseKind::CallResult(result) = response.kind {
            return Some(result.result)
        }

        Option::<Vec<u8>>::None
    }

    pub fn send_headers(& mut self, headers: &Vec<BlockHeader>, st_slot: u64, end_slot: u64) -> Result<(), Box<dyn std::error::Error>>{
        println!("Send headers, #headers = {} ", headers.len());

        if headers.len() == 0 {
            self.last_slot = end_slot;
            return Ok(());
        }

        let headers_filename = format!("headers_slots_{}_{}.json",
                                       st_slot,
                                       end_slot);
        let header_path = Path::new(&self.dir_path).join(headers_filename);
        let headers_json_str = serde_json::to_string(&headers)?;

        let mut file = File::create(header_path)?;
        file.write_all(headers_json_str.as_bytes())?;

        let rt = Runtime::new()?;
        let handle = rt.handle();

        self.last_slot = end_slot;

        let access_key_query_response = handle.block_on(self.client
            .call(methods::query::RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id: self.signer.account_id.clone(),
                    public_key: self.signer.public_key.clone(),
                },
            }))?;

        for header in headers {
            let current_nonce = self.get_current_nonce();
            let transaction = Transaction {
                signer_id: self.signer.account_id.clone(),
                public_key: self.signer.public_key.clone(),
                nonce: current_nonce + 1,
                receiver_id: self.contract_account.clone(),
                block_hash: access_key_query_response.block_hash,
                actions: vec![Action::FunctionCall(FunctionCallAction {
                    method_name: "submit_header".to_string(),
                    args: header.try_to_vec().unwrap(),
                    gas: 100_000_000_000_000, // 100 TeraGas
                    deposit: 0,
                })],
            };

            println!("{:?}", header);

            let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
                signed_transaction: transaction.sign(&self.signer),
            };

            for _ in 1..5 {
                if let Ok(_response) = handle.block_on(self.client.call(&request)) {
                    break;
                }
            }
        }

        Ok(())
    }

    fn get_current_nonce(& self) -> Nonce {
        let rt = Runtime::new().unwrap();
        let handle = rt.handle();

        let access_key_query_response = handle.block_on(self.client
            .call(methods::query::RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id: self.signer.account_id.clone(),
                    public_key: self.signer.public_key.clone(),
                },
            })).unwrap();

        let current_nonce = match access_key_query_response.kind {
            QueryResponseKind::AccessKey(access_key) => access_key.nonce,
            _ => Err("failed to extract current nonce").unwrap(),
        };

        current_nonce
    }
}