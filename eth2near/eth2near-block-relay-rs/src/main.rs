use clap::{arg, Parser};
use eth2_to_near_relay::eth2near_relay::Eth2NearRelay;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::QueryRequest;
use near_primitives::transaction::{Action, FunctionCallAction, Transaction};
use near_primitives::borsh::BorshSerialize;
use near_primitives::types::AccountId;
use serde_json::{from_slice, Value};
use serde_json::json;
use std::string::String;
use env_logger::init;
use eth_types::eth2::ExtendedBeaconBlockHeader;
use near_crypto::InMemorySigner;
use types::SecretKey;
use eth2_to_near_relay::beacon_rpc_client::BeaconRPCClient;
use tokio::runtime::Runtime;
use eth_types::eth2::SyncCommittee;

#[derive(Parser,Default,Debug)]
#[clap(version, about="Eth2 to Near Relay")]
struct Arguments {
    #[clap(long, default_value_t = String::from("https://lodestar-kiln.chainsafe.io"))]
    /// endpoint to full node of Eth2 Beacon chain with Light Client API
    eth_endpoint: String,

    #[clap(long, default_value_t = String::from("https://rpc.kiln.themerge.dev"))]
    /// endpoint for the ethereum full node which support Eth1 RPC API
    eth1_endpoint: String,

    #[clap(long="total-submit-headers", default_value_t = 4)]
    /// the max number of headers submitted in one bunch to eth client
    total_submit_headers: u32,

    #[clap(long, default_value_t = String::from("https://rpc.testnet.near.org"))]
    /// endpoint for full node on NEAR chain
    near_endpoint: String,

    #[clap(long, default_value_t = String::from("olga24912.testnet"))]
    /// Account id from which relay make requests
    signer_account_id: String,

    #[clap(long, default_value_t = String::from("/home/olga/.near-credentials/testnet/olga24912.testnet.json"))]
    /// Path to the file with secret key for signer account
    path_to_signer_secret_key: String,

    #[clap(long, default_value_t = String::from("eth2_1.test1-dev.testnet"))]
    /// Eth client on NEAR account id
    contract_account_id: String,

    #[clap(long, default_value_t = 812544)]
    /// Tmp flag TODO: remove
    start_slot: u64,

    #[clap(long, default_value_t = String::from("./light_client_updates_out"))]
    /// Tmp output dir TODO remove
    output_dir: String,
}

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

fn init_contract(signer: &InMemorySigner, args: &Arguments, client: &JsonRpcClient) -> Result<(), Box<dyn std::error::Error>> {
    let contract_account = args.contract_account_id.parse().unwrap();

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
        //max_submitted_blocks_by_account: args.total_submit_headers,
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
        signed_transaction: transaction.sign(signer),
    };

    let response = handle.block_on(client.call(request))?;
    println!("response: {:#?}", response);

    Ok(())
}

//#[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();

    let client = JsonRpcClient::connect(&args.near_endpoint);

    println!("path: {}", args.path_to_signer_secret_key);

    let signer_account_id = args.signer_account_id.parse().unwrap();
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&args.path_to_signer_secret_key).expect("Unable to read file")).unwrap();
    let signer_secret_key = serde_json::to_string(&v["private_key"]).unwrap();
    let signer_secret_key = &signer_secret_key[1..signer_secret_key.len() - 1];

    println!("{}: {}", signer_account_id, signer_secret_key);

    let signer = near_crypto::InMemorySigner::from_secret_key(signer_account_id, signer_secret_key.parse().unwrap());
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

    let contract_account = args.contract_account_id.parse().unwrap();

    let transaction = Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce: current_nonce + 1,
        receiver_id: contract_account,
        block_hash: access_key_query_response.block_hash,
        actions: vec![Action::FunctionCall(FunctionCallAction {
            method_name: "initialized".to_string(),
            args: json!({})
                .to_string()
                .into_bytes(),
            gas: 100_000_000_000_000, // 100 TeraGas
            deposit: 0,
        })],
    };

    let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
        signed_transaction: transaction.sign(&signer),
    };

    let response =  handle.block_on(client.call(request))?;

    println!("response: {:#?}", response);
    //let contract_account = args.contract_account_id.parse().unwrap();

    //init_contract(&signer, &argsg , &client)?;

    let mut eth2near_relay = Eth2NearRelay::init(&args.eth_endpoint, &args.eth1_endpoint, args.start_slot,
                                                 args.output_dir, args.total_submit_headers,
                                                  &args.near_endpoint, &args.signer_account_id,
                                                  &args.path_to_signer_secret_key, &args.contract_account_id);
    //Ok(eth2near_relay.run())
    Ok(())
}