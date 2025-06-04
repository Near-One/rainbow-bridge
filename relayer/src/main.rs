use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use relayer::{config::Config, relay::EthRelayer};
use tracing::level_filters::LevelFilter;

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
    GenerateConfig {
        #[arg(short, long, default_value = "relayer.toml")]
        output: PathBuf,
    },
    ValidateConfig,
    Run,
}
fn setup_logging(level: &str, json: bool) -> Result<()> {
    let level = match level {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => return Err(color_eyre::eyre::eyre!("Invalid log level: {}", level)),
    };

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false);
    if json {
        subscriber.json().init()
    } else {
        subscriber.init()
    }
    Ok(())
}

async fn handle_command(command: Commands, config_path: Option<PathBuf>) -> Result<()> {
    match command {
        Commands::GenerateConfig { output } => {
            std::fs::write(&output, Config::example_toml()?)?;
            println!("📝 Example configuration written to {}", output.display());
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
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    handle_command(cli.command.unwrap_or(Commands::Run), cli.config).await
}
