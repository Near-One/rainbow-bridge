use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use color_eyre::Result;
use relayer::{
    config::{Config, Network},
    relay::EthRelayer,
};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt};

/// Network selection for config generation
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum NetworkArg {
    /// Sepolia testnet + NEAR testnet
    #[default]
    Testnet,
    /// Ethereum mainnet + NEAR mainnet
    Mainnet,
}

impl From<NetworkArg> for Network {
    fn from(arg: NetworkArg) -> Self {
        match arg {
            NetworkArg::Testnet => Network::Testnet,
            NetworkArg::Mainnet => Network::Mainnet,
        }
    }
}

#[derive(Parser)]
#[command(
    name = "eth-relayer",
    about = "Ethereum to NEAR light client relayer",
    version
)]
struct Cli {
    #[arg(short, long)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate an example configuration file
    GenerateConfig {
        /// Output file path
        #[arg(short, long, default_value = "relayer.toml")]
        output: PathBuf,
        /// Target network for default values
        #[arg(short, long, default_value = "testnet")]
        network: NetworkArg,
    },
    /// Validate the configuration file
    ValidateConfig,
    /// Run the relayer continuously
    Run,
    /// Run a single relay job and exit
    RunJob,
    /// Init Eth Light Client contract
    Init,
}

fn setup_logging(level: &str, json: bool) -> Result<()> {
    // Create an environment filter with the specified level
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    // Create the indicatif layer for progress bars
    let indicatif_layer = IndicatifLayer::new();

    // Create fmt layer with indicatif writer to prevent progress bar interference
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(indicatif_layer.get_stderr_writer())
        .with_target(false);

    // Build and initialize the subscriber
    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(indicatif_layer);

    if json {
        registry.with(fmt_layer.json()).init();
    } else {
        registry.with(fmt_layer).init();
    }

    Ok(())
}

async fn handle_command(command: Commands, config_path: Option<PathBuf>) -> Result<()> {
    match command {
        Commands::GenerateConfig { output, network } => {
            let network: Network = network.into();
            std::fs::write(&output, Config::example_toml_for_network(network)?)?;
            println!(
                "Example {:?} configuration written to {}",
                network,
                output.display()
            );
        }
        Commands::ValidateConfig => {
            let config = Config::load(config_path)?;
            config.validate()?;
            println!("✅ Configuration is valid");
            config.print_summary();
        }
        Commands::Run => {
            let config = Config::load(config_path)?;
            config.validate()?;
            setup_logging(&config.logging.level, config.logging.json)?;
            config.print_summary();

            EthRelayer::new(config).await?.run().await?;
        }
        Commands::RunJob => {
            let config = Config::load(config_path)?;
            config.validate()?;
            setup_logging(&config.logging.level, config.logging.json)?;
            config.print_summary();

            EthRelayer::new(config).await?.run_job().await?;
        }
        Commands::Init => {
            let config = Config::load(config_path)?;
            config.validate()?;
            setup_logging(&config.logging.level, config.logging.json)?;
            config.print_summary();

            EthRelayer::new(config).await?.init_eth_client().await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    handle_command(cli.command.unwrap_or(Commands::Run), cli.config).await
}
