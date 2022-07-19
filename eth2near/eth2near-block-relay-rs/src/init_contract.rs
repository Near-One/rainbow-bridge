use eth_types::eth2::ExtendedBeaconBlockHeader;
use eth_types::eth2::SyncCommittee;
use near_jsonrpc_client::{JsonRpcClient, methods};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::types::BlockReference;
use near_primitives::borsh::BorshSerialize;
use near_sdk::AccountId;
use serde_json::Value;
use tokio::runtime::Runtime;
use crate::beacon_rpc_client::BeaconRPCClient;

#[derive(BorshSerialize)]
pub struct InitInput {
    pub network: String,
    pub finalized_header: ExtendedBeaconBlockHeader,
    pub current_sync_committee: SyncCommittee,
    pub next_sync_committee: SyncCommittee,
    pub validate_updates: bool,
    pub verify_bls_signatures: bool,
    pub hashes_gc_threshold: u64,
    //pub max_submitted_blocks_by_account: u32,
    //pub min_storage_balance_for_submitter: near_sdk::Balance,
    pub trusted_signer: Option<AccountId>,
}

pub fn init_contract(near_endpoint: &str, signer_account_id: &str, path_to_signer_secret_key: &str, contract_account_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = JsonRpcClient::connect(near_endpoint);

    let signer_account_id = signer_account_id.parse().unwrap();
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path_to_signer_secret_key).expect("Unable to read file")).unwrap();
    let signer_secret_key = serde_json::to_string(&v["private_key"]).unwrap();
    let signer_secret_key = &signer_secret_key[1..signer_secret_key.len() - 1];

    println!("{}: {}", signer_account_id, signer_secret_key);

    let signer = near_crypto::InMemorySigner::from_secret_key(signer_account_id, signer_secret_key.parse().unwrap());

    let contract_account = contract_account_id.parse().unwrap();

    let rt = Runtime::new().unwrap();
    let handle = rt.handle();

    let access_key_query_response = handle.block_on(client
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        }))?;

    let current_nonce = match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ => Err("failed to extract current nonce")?,
    };

    println!("current_nonce: {}", current_nonce);

    let beacon_rpc_client = BeaconRPCClient::default();
    println!("beacon rpc client");
    let light_client_update = beacon_rpc_client.get_light_client_update(99)?;
    println!("light client update: {:?}", light_client_update);

    let finalized_header : ExtendedBeaconBlockHeader = ExtendedBeaconBlockHeader::from(light_client_update.finality_update.header_update);

    println!("finalized header: {:?}", finalized_header);

    let next_sync_committee = light_client_update.sync_committee_update.unwrap().next_sync_committee;

    let prev_light_client_update = beacon_rpc_client.get_light_client_update(98)?;
    let current_sync_committee = prev_light_client_update.sync_committee_update.unwrap().next_sync_committee;

    println!("Before transactions");
    let init_input = InitInput {
        network: String::from("kiln"),
        finalized_header: finalized_header,
        current_sync_committee: current_sync_committee,
        next_sync_committee: next_sync_committee,
        validate_updates: true,
        verify_bls_signatures: false,
        hashes_gc_threshold: 51000,
        //max_submitted_blocks_by_account: 30000,
        //min_storage_balance_for_submitter: near_sdk::ONE_NEAR,
        trusted_signer: Option::<AccountId>::None,
    };

    let transaction = Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce: current_nonce + 1,
        receiver_id: contract_account,
        block_hash: access_key_query_response.block_hash,
        actions: vec![Action::FunctionCall(FunctionCallAction {
            method_name: "init".to_string(),
            args: init_input.try_to_vec()?,
            gas: 100_000_000_000_000, // 100 TeraGas
            deposit: 0,
        })],
    };

    let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
        signed_transaction: transaction.sign(&signer),
    };

    let response = handle.block_on(client.call(request))?;
    println!("response: {:#?}", response);

    Ok(())
}