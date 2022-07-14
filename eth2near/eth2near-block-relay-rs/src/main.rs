use clap::Parser;

#[derive(Parser,Default,Debug)]
#[clap(version, about="Eth2 to Near Relay")]
struct Arguments {
    eth_node_url: std::string::String,
}

fn main() {
    let args = Arguments::parse();
    println!("{:?}", args);
}