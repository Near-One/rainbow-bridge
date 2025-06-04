use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use eth2_utility::types::ClientMode;
use near_crypto::{InMemorySigner, SecretKey};
use relayer::{beacon::BeaconLightClientService, execution::ExecutionClient, near::NearContract};
use tokio::time::sleep;
use tracing::{debug, error, info, level_filters::LevelFilter, warn};

use relayer::config::Config;

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

#[derive(Debug)]
enum RelayResult {
    Submitted,
    NoWork,
    Error(color_eyre::Report),
}

pub struct EthRelayer {
    beacon_client: BeaconLightClientService,
    execution_client: ExecutionClient,
    near_contract: NearContract,
    config: Config,
}

impl EthRelayer {
    pub async fn new(config: Config) -> Result<Self> {
        let beacon_client = BeaconLightClientService::new(&config.beacon.endpoint)?;
        let execution_client = ExecutionClient::new(&config.execution.endpoint)?;
        let near_contract = Self::create_near_contract(&config).await?;

        Ok(Self {
            beacon_client,
            execution_client,
            near_contract,
            config,
        })
    }

    async fn create_near_contract(config: &Config) -> Result<NearContract> {
        let secret_key: SecretKey = std::fs::read_to_string(&config.near.secret_key_path)?
            .trim()
            .parse()?;
        let (contract_account_id, signer_account_id) = config.parse_near_accounts()?;
        let signer = InMemorySigner::from_secret_key(signer_account_id, secret_key);
        let client = near_fetch::Client::new(&config.near.endpoint);

        Ok(NearContract::new(contract_account_id, signer, client))
    }

    pub async fn run(&self) -> Result<()> {
        info!("🚀 Starting ETH to NEAR relayer");

        for iteration in 1.. {
            if self
                .config
                .relayer
                .max_iterations
                .map_or(false, |max| iteration > max)
            {
                info!("Reached maximum iterations, stopping");
                break;
            }

            info!("=== Relay Loop {} ===", iteration);

            let result = self.run_iteration().await;
            let sleep_secs = match &result {
                RelayResult::Submitted => {
                    info!("✅ Operation completed");
                    self.config.relayer.submission_sleep_secs
                }
                RelayResult::NoWork => {
                    info!("⏭️ No work to do");
                    self.config.relayer.sync_sleep_secs
                }
                RelayResult::Error(e) => {
                    error!("❌ Error: {}", e);
                    self.config.relayer.sync_sleep_secs
                }
            };

            sleep(Duration::from_secs(sleep_secs)).await;
        }
        Ok(())
    }

    async fn run_iteration(&self) -> RelayResult {
        // Early return pattern - convert all errors to RelayResult::Error
        let mode = match self.get_mode_if_synced().await {
            Ok(mode) => mode,
            Err(e) => return RelayResult::Error(e),
        };

        let result = match mode {
            ClientMode::SubmitLightClientUpdate => {
                info!("📡 Light Client Update Mode");
                self.try_submit_light_client_update().await
            }
            ClientMode::SubmitHeader => {
                info!("🔗 Submit Header Mode");
                self.try_submit_headers().await
            }
        };

        result.unwrap_or_else(RelayResult::Error)
    }

    async fn get_mode_if_synced(&self) -> Result<ClientMode> {
        if self.beacon_client.is_syncing().await? {
            info!("⏳ Beacon node is syncing, waiting...");
            return Err(color_eyre::eyre::eyre!("Beacon node is syncing"));
        }
        info!("✅ All clients are synchronized");
        self.near_contract.get_client_mode().await
    }

    async fn try_submit_light_client_update(&self) -> Result<RelayResult> {
        if self.config.relayer.dry_run {
            info!("🧪 DRY RUN: Would submit light client update");
            return Ok(RelayResult::Submitted);
        }

        let near_slot = self.near_contract.get_finalized_beacon_block_slot().await?;
        let eth_slot = self.beacon_client.get_last_finalized_slot().await?;
        info!("Finalized slots - NEAR: {}, ETH: {}", near_slot, eth_slot);

        if !self.should_update(near_slot, eth_slot) {
            debug!(
                "No update needed: ETH slot {} <= NEAR slot {} or insufficient difference",
                eth_slot, near_slot
            );
            return Ok(RelayResult::NoWork);
        }

        let update = self.fetch_update_for_slots(near_slot, eth_slot).await?;
        self.near_contract
            .submit_light_client_update(update)
            .await?;

        Ok(RelayResult::Submitted)
    }

    async fn try_submit_headers(&self) -> Result<RelayResult> {
        if self.config.relayer.dry_run {
            info!("🧪 DRY RUN: Would submit execution headers");
            return Ok(RelayResult::Submitted);
        }

        let last_block = self.near_contract.get_last_block_number().await?;
        let max_block = self.get_max_block().await?;

        if max_block <= last_block {
            debug!(
                "No new blocks to submit (last: {}, max: {})",
                last_block, max_block
            );
            return Ok(RelayResult::NoWork);
        }

        let start_block = last_block + 1;
        let end_block =
            (start_block + self.config.relayer.headers_batch_size as u64 - 1).min(max_block);
        info!(
            "Fetching execution headers for blocks {} to {}",
            start_block, end_block
        );

        let mut headers = self
            .execution_client
            .fetch_block_range(start_block..=end_block)
            .await?;

        if headers.is_empty() {
            warn!(
                "No headers fetched for range {}..={}",
                start_block, end_block
            );
            return Ok(RelayResult::NoWork);
        }

        info!("Fetched {} headers, submitting to NEAR", headers.len());
        headers.reverse();
        self.near_contract
            .submit_execution_headers(&headers)
            .await?;

        Ok(RelayResult::Submitted)
    }

    async fn fetch_update_for_slots(
        &self,
        near_slot: u64,
        eth_slot: u64,
    ) -> Result<eth_types::eth2::LightClientUpdate> {
        let near_period = BeaconLightClientService::get_period_for_slot(near_slot);
        let eth_period = BeaconLightClientService::get_period_for_slot(eth_slot);

        if eth_period == near_period {
            self.beacon_client.fetch_finality_update().await
        } else {
            self.beacon_client
                .fetch_period_update(near_period + 1)
                .await
        }
    }

    fn should_update(&self, near_slot: u64, eth_slot: u64) -> bool {
        eth_slot > near_slot
            && (eth_slot - near_slot) >= (32 * self.config.relayer.update_interval_epochs)
    }

    async fn get_max_block(&self) -> Result<u64> {
        match self
            .near_contract
            .get_unfinalized_tail_block_number()
            .await?
        {
            Some(tail) => Ok(tail - 1),
            None => {
                let slot = self.near_contract.get_finalized_beacon_block_slot().await?;
                self.beacon_client.get_block_number_for_slot(slot).await
            }
        }
    }
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
