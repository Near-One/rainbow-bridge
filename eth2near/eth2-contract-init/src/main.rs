use clap::Parser;
use contract_wrapper::contract_wrapper_trait::ContractWrapper;
use contract_wrapper::eth_client_contract::EthClientContract;
use contract_wrapper::near_contract_wrapper::NearContractWrapper;
use eth2_contract_init::config::Config;
use eth2_contract_init::init_contract::init_contract;
use eth2near_logger::SimpleLogger;
use log::LevelFilter;
use std::string::String;

#[derive(Parser, Default, Debug)]
#[clap(version, about = "ETH2 contract initialization")]
struct Arguments {
    #[clap(short, long)]
    /// Path to config file
    config: String,

    #[clap(long, default_value_t = String::from("info"))]
    /// Log level (trace, debug, info, warn, error)
    log_level: String,
}

fn get_eth_contract_wrapper(config: &Config) -> Box<dyn ContractWrapper> {
    Box::new(NearContractWrapper::new(
        &config.near_endpoint,
        &config.signer_account_id,
        &config.path_to_signer_secret_key,
        &config.contract_account_id,
        None,
    ))
}

fn init_log(args: &Arguments, config: &Config) {
    let log_level_filter = match args.log_level.as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    let mut path_to_log_file = "./eth2-contract-init.log".to_string();
    if let Some(out_dir) = config.clone().output_dir {
        path_to_log_file = out_dir.clone() + "/" + "eth2-contract-init.log";
        std::fs::create_dir_all(out_dir).expect("Error during creation output dirs in path");
    }

    log::set_boxed_logger(Box::new(SimpleLogger::new(path_to_log_file)))
        .map(|()| log::set_max_level(log_level_filter))
        .expect("Error during logger creation");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();
    let config = Config::load_from_toml(
        args.config
            .clone()
            .try_into()
            .expect("Incorrect config path"),
    );
    init_log(&args, &config);

    let mut eth_client_contract = EthClientContract::new(get_eth_contract_wrapper(&config));
    init_contract(&config, &mut eth_client_contract).expect("Error on contract initialization");

    Ok(())
}
