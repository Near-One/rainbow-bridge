use clap::{Parser, ArgAction};
use eth2_to_near_relay::eth2near_relay::Eth2NearRelay;
use std::string::String;
use eth2_to_near_relay::init_contract::init_contract;

#[derive(Parser,Default,Debug)]
#[clap(version, about="Eth2 to Near Relay")]
struct Arguments {
    #[clap(long, default_value_t = String::from("https://lodestar-kiln.chainsafe.io"))]
    /// endpoint to full node of Eth2 Beacon chain with Light Client API
    eth_endpoint: String,

    #[clap(long, default_value_t = String::from("https://rpc.kiln.themerge.dev"))]
    /// endpoint for the ethereum full node which support Eth1 RPC API
    eth1_endpoint: String,

    #[clap(long="total-submit-headers", default_value_t = 1)]
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

    #[clap(long, action = ArgAction::SetTrue)]
    /// The eth contract on Near will be initialized
    init_contract: bool,

    #[clap(long, default_value_t = String::from("eth2_1.test1-dev.testnet"))]
    /// Eth client on NEAR account id
    contract_account_id: String,

    #[clap(long, default_value_t = 817376)]
    /// Tmp flag TODO: remove
    start_slot: u64,

    #[clap(long, default_value_t = String::from("./light_client_updates_out"))]
    /// Tmp output dir TODO remove
    output_dir: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();

    if args.init_contract == true {
        init_contract(&args.near_endpoint, &args.signer_account_id, &args.path_to_signer_secret_key, &args.contract_account_id)?;
    }

    let mut eth2near_relay = Eth2NearRelay::init(&args.eth_endpoint, &args.eth1_endpoint, args.start_slot,
                                                 args.output_dir, args.total_submit_headers,
                                                  &args.near_endpoint, &args.signer_account_id,
                                                  &args.path_to_signer_secret_key, &args.contract_account_id);
    Ok(eth2near_relay.run())
}