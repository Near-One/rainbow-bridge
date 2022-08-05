extern crate core;

use clap::{ArgAction, Parser};
use contract_wrapper::contract_wrapper_trait::ContractWrapper;
use contract_wrapper::eth_client_contract_trait::EthClientContractTrait;
use contract_wrapper::near_contract_wrapper::NearContractWrapper;
use contract_wrapper::{dao_contract, dao_eth_client_contract, eth_client_contract};
use eth2_to_near_relay::config::Config;
use eth2_to_near_relay::eth2near_relay::Eth2NearRelay;
use eth2_to_near_relay::init_contract::init_contract;
use eth2_to_near_relay::logger::SimpleLogger;
use log::LevelFilter;
use std::string::String;

#[derive(Parser, Default, Debug)]
#[clap(version, about = "Eth2 to Near Relay")]
struct Arguments {
    #[clap(short, long)]
    /// Path to config file
    config: String,

    #[clap(long, action = ArgAction::SetTrue)]
    /// The eth contract on Near will be initialized
    init_contract: bool,

    #[clap(long, action = ArgAction::SetTrue)]
    /// Relay will be registered in contract
    register_relay: bool,

    #[clap(long, default_value_t = String::from("info"))]
    /// Log level (trace, debug, info, warn, error)
    log_level: String,

    #[clap(long, action = ArgAction::SetTrue)]
    /// Enable binary search for last slot ETH block on NEAR
    enable_binary_search: bool,
}

fn get_eth_contract_wrapper(config: &Config) -> Box<dyn ContractWrapper> {
    Box::new(NearContractWrapper::new(
        &config.near_endpoint,
        &config.signer_account_id,
        &config.path_to_signer_secret_key,
        &config.contract_account_id,
    ))
}

fn get_dao_contract_wrapper(config: &Config) -> Box<dyn ContractWrapper> {
    let dao_contract_account_id = config.dao_contract_account_id.clone();

    Box::new(NearContractWrapper::new(
        &config.near_endpoint,
        &config.signer_account_id,
        &config.path_to_signer_secret_key,
        &dao_contract_account_id.unwrap(),
    ))
}

fn get_eth_client_contract(config: &Config) -> Box<dyn EthClientContractTrait> {
    let eth_contract_wrapper = get_eth_contract_wrapper(config);
    let eth_client = eth_client_contract::EthClientContract::new(eth_contract_wrapper);

    match config.contract_type.as_str() {
        "dao" => Box::new(dao_eth_client_contract::DaoEthClientContract::new(
            eth_client,
            dao_contract::DAOContract::new(get_dao_contract_wrapper(config)),
        )),
        _ => Box::new(eth_client),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();
    let log_level_filter = match args.log_level.as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    log::set_boxed_logger(Box::new(SimpleLogger))
        .map(|()| log::set_max_level(log_level_filter))
        .unwrap();
    let config = Config::load_from_toml(args.config.try_into().unwrap());

    if args.init_contract {
        init_contract(&config, get_eth_contract_wrapper(&config)).unwrap();
    }

    let mut eth2near_relay = Eth2NearRelay::init(
        &config,
        get_eth_client_contract(&config),
        args.enable_binary_search,
        args.register_relay,
    );

    eth2near_relay.run();
    Ok(())
}
