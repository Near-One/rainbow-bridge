extern crate core;

use clap::{Parser, ArgAction};
use eth2_to_near_relay::eth2near_relay::Eth2NearRelay;
use std::string::String;
use contract_wrapper::contract_wrapper_trait::ContractWrapper;
use contract_wrapper::near_contract_wrapper::NearContractWrapper;
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

    #[clap(long, default_value_t = String::from("info"))]
    /// Log level (trace, info, warn, error)
    log_level: String, 
}

fn get_contract_wrapper(config: &Config) -> Box<dyn ContractWrapper> {
    match config.contract_type.as_str() {
        "near" => Box::new(NearContractWrapper::new(&config.near_endpoint, &config.signer_account_id,
                                                    &config.path_to_signer_secret_key,
                                                    &config.contract_account_id)),
        _ => Box::new(NearContractWrapper::new(&config.near_endpoint, &config.signer_account_id,
                                               &config.path_to_signer_secret_key,
                                               &config.contract_account_id))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();
    let log_level_filter = match args.log_level.as_str() {
        "trace" => LevelFilter::Trace,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info
    };

    log::set_boxed_logger(Box::new(SimpleLogger)).map(|()| log::set_max_level(log_level_filter)).unwrap();
    let config = Config::load_from_toml(args.config.try_into().unwrap());

    if args.init_contract == true {
        init_contract(&config, get_contract_wrapper(&config)).unwrap();
    }

    let mut eth2near_relay = Eth2NearRelay::init(&config, get_contract_wrapper(&config));

    Ok(eth2near_relay.run())
}