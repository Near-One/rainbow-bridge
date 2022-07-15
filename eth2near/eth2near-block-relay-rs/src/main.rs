use clap::Parser;
use std::string::String;
use eth2_to_near_relay::eth2near_relay::Eth2NearRelay;

#[derive(Parser,Default,Debug)]
#[clap(version, about="Eth2 to Near Relay")]
struct Arguments {
    #[clap(long, default_value_t = String::from("https://lodestar-kiln.chainsafe.io"))]
    /// endpoint to full node of Eth2 Beacon chain with Light Client API
    eth_node_url: String,

    #[clap(long, default_value_t = 823648)]
    ///Tmp flag TODO: remove
    start_slot: u64,

    #[clap(long, default_value_t = String::from("./light_client_updates_out"))]
    ///Tmp output dir TODO remove
    output_dir: String,
}

fn main() {
    let args = Arguments::parse();
    let mut eth2near_relay = Eth2NearRelay::init(&args.eth_node_url, args.start_slot, args.output_dir);
    eth2near_relay.run();
}