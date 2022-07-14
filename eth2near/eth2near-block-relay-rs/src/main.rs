use clap::Parser;
use std::string::String;

#[derive(Parser,Default,Debug)]
#[clap(version, about="Eth2 to Near Relay")]
struct Arguments {
    #[clap(long, default_value_t = String::from("https://lodestar-kiln.chainsafe.io"))]
    /// endpoint to full node of Eth2 Beacon chain with Light Client API
    eth_node_url: String,
}

fn main() {
    let args = Arguments::parse();
    println!("{:?}", args);
}