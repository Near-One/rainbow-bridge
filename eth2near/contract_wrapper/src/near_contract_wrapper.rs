use crate::contract_wrapper_trait::ContractWrapper;
use near_crypto::InMemorySigner;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::{AccountId, BlockReference, Finality, FunctionArgs};
use near_primitives::views::{FinalExecutionOutcomeView, QueryRequest};
use near_sdk::{Balance, Gas};
use serde_json::Value;
use std::error::Error;
use std::string::String;
use std::vec::Vec;
use tokio::runtime::Runtime;
use crate::utils::trim_quotes;
use tokio::task;
use std::time::Duration;

pub const MAX_GAS: Gas = Gas(Gas::ONE_TERA.0 * 300);

pub struct NearContractWrapper {
    client: JsonRpcClient,
    contract_account: AccountId,
    signer: InMemorySigner,
}

impl NearContractWrapper {
    pub fn new_with_raw_secret_key(
        near_endpoint: &str,
        account_id: &str,
        signer_secret_key: &str,
        contract_account_id: &str,
    ) -> NearContractWrapper {
        let signer_account_id = account_id.parse().unwrap();
        let client = JsonRpcClient::connect(near_endpoint);
        let contract_account = contract_account_id.parse().unwrap();

        let signer =
            InMemorySigner::from_secret_key(signer_account_id, signer_secret_key.parse().unwrap());

        NearContractWrapper {
            client,
            contract_account,
            signer,
        }
    }

    pub fn new(
        near_endpoint: &str,
        account_id: &str,
        path_to_signer_secret_key: &str,
        contract_account_id: &str,
    ) -> NearContractWrapper {
        let v: Value = serde_json::from_str(
            &std::fs::read_to_string(path_to_signer_secret_key).expect("Unable to read file"),
        )
        .unwrap();
        let signer_secret_key = trim_quotes(serde_json::to_string(&v["private_key"]).unwrap());

        Self::new_with_raw_secret_key(
            near_endpoint,
            account_id,
            &signer_secret_key,
            contract_account_id,
        )
    }
}

impl ContractWrapper for NearContractWrapper {
    fn get_account_id(&self) -> AccountId {
        self.contract_account.clone()
    }

    fn call_view_function(
        &self,
        method_name: String,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let rt = Runtime::new()?;
        let handle = rt.handle();

        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id: self.contract_account.clone(),
                method_name,
                args: FunctionArgs::from(args),
            },
        };

        let response = handle.block_on(self.client.call(request))?;

        if let QueryResponseKind::CallResult(result) = response.kind {
            Ok(result.result)
        } else {
            Err("view method doesn't return any result")?
        }
    }

    fn call_change_method_batch(
        &self,
        method_name: Vec<String>,
        args: Vec<Vec<u8>>,
        deposit: Option<Vec<Balance>>,
        gas: Option<Gas>,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn Error>> {
        let rt = Runtime::new()?;

        let access_key_query_response =
            rt.block_on(self.client.call(methods::query::RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id: self.signer.account_id.clone(),
                    public_key: self.signer.public_key.clone(),
                },
            }))?;

        let current_nonce = match access_key_query_response.kind {
            QueryResponseKind::AccessKey(access_key) => access_key.nonce,
            _ => Err("failed to extract current nonce")?,
        };

        let num_blocks_in_batch = method_name.len() as u64;

        if num_blocks_in_batch == 0 {
            return Err(Box::new(crate::errors::TryToSubmitZeroHeaderError));
        }

        let attached_gas_per_promise_in_batch = gas.unwrap_or(MAX_GAS) / num_blocks_in_batch;
        let mut actions = Vec::new();

        for i in 0..method_name.len() {
            actions.push(Action::FunctionCall(FunctionCallAction {
                method_name: method_name[i].clone(),
                args: args[i].clone(),
                gas: attached_gas_per_promise_in_batch.0,
                deposit: deposit.as_ref().map(|d| d[i]).unwrap_or(0),
            }));
        }

        let transaction = Transaction {
            signer_id: self.signer.account_id.clone(),
            public_key: self.signer.public_key.clone(),
            nonce: current_nonce + 1,
            receiver_id: self.contract_account.clone(),
            block_hash: access_key_query_response.block_hash,
            actions,
        };

        let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
            signed_transaction: transaction.sign(&self.signer),
        };

        let request_result = rt.block_on(async_std::future::timeout(
            std::time::Duration::from_secs(600),
            self.client.call(&request)))?;
        Ok(request_result?)
    }

    fn call_change_method(
        &self,
        method_name: String,
        args: Vec<u8>,
        deposit: Option<Balance>,
        gas: Option<Gas>,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn Error>> {
        self.call_change_method_batch(
            vec![method_name],
            vec![args],
            Some(vec![deposit.unwrap_or(0)]),
            gas,
        )
    }
}
