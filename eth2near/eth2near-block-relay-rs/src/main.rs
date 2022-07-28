use clap::{Parser, ArgAction};
use eth2_to_near_relay::eth2near_relay::Eth2NearRelay;
use std::string::String;
use log::{LevelFilter};
use eth2_to_near_relay::init_contract::init_contract;
use eth2_to_near_relay::logger::SimpleLogger;
use eth2_to_near_relay::config::Config;

#[derive(Parser,Default,Debug)]
#[clap(version, about="Eth2 to Near Relay")]
struct Arguments {
    #[clap(short, long)]
    /// Path to config file
    config: String,

    #[clap(long, action = ArgAction::SetTrue)]
    /// The eth contract on Near will be initialized
    init_contract: bool,

    #[clap(long, default_value_t = 956937)]
    /// Tmp flag TODO: remove
    start_slot: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log::set_boxed_logger(Box::new(SimpleLogger)).map(|()| log::set_max_level(LevelFilter::Trace)).unwrap();
    let args = Arguments::parse();
    let config = Config::load_from_toml(args.config.try_into().unwrap());

    if args.init_contract == true {
        init_contract(&config, args.start_slot).unwrap();
    }
    
    let mut eth2near_relay = Eth2NearRelay::init(&config, args.start_slot);

    Ok(eth2near_relay.run())
}