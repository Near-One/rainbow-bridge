use clap::Parser;
use std::string::String;
use eth2_to_near_relay::eth2near_relay::Eth2NearRelay;
use futures;
use tokio;

#[derive(Parser,Default,Debug)]
#[clap(version, about="Eth2 to Near Relay")]
struct Arguments {
    #[clap(long, default_value_t = String::from("https://lodestar-kiln.chainsafe.io"))]
    /// endpoint to full node of Eth2 Beacon chain with Light Client API
    eth_node_url: String,
}

#[tokio::main]
async fn main() {
    let args = Arguments::parse();
    let mut eth2near_relay = Eth2NearRelay::init(&args.eth_node_url);
    let future_relay = eth2near_relay.run();
    futures::executor::block_on(future_relay);
}