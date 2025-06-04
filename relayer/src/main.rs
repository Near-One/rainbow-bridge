use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use color_eyre::{Result, eyre::Context};
use eth2_utility::types::ClientMode;
use near_crypto::{InMemorySigner, SecretKey};
use relayer::{beacon::BeaconLightClientService, execution::ExecutionClient, near::NearContract};
use tokio::time::sleep;
use tracing::{debug, error, info, level_filters::LevelFilter};

use relayer::config::Config;

#[derive(Parser)]
#[command(name = "eth-relayer")]
#[command(about = "Ethereum to NEAR light client relayer")]
#[command(version)]
struct Cli {
    /// Path to configuration file
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
    },
    /// Validate configuration without running the relayer
    ValidateConfig,
    /// Run the relayer
    Run,
}

#[derive(Debug)]
enum RelayResult {
    Submitted,
    NoWorkToDo,
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
        let beacon_client = BeaconLightClientService::new(&config.beacon.endpoint)
            .wrap_err("Failed to create beacon light client service")?;

        let execution_client = ExecutionClient::new(&config.execution.endpoint)
            .wrap_err("Failed to create execution client")?;

        let near_contract = Self::create_near_contract(&config).await?;

        Ok(Self {
            beacon_client,
            execution_client,
            near_contract,
            config,
        })
    }

    async fn create_near_contract(config: &Config) -> Result<NearContract> {
        let secret_key_str =
            std::fs::read_to_string(&config.near.secret_key_path).wrap_err_with(|| {
                format!(
                    "Failed to read secret key from {}",
                    config.near.secret_key_path.display()
                )
            })?;

        let secret_key: SecretKey = secret_key_str
            .trim()
            .parse()
            .wrap_err("Failed to parse secret key")?;

        let (contract_account_id, signer_account_id) = config
            .parse_near_accounts()
            .wrap_err("Failed to parse NEAR account IDs")?;

        let signer = InMemorySigner::from_secret_key(signer_account_id, secret_key);
        let near_client = near_fetch::Client::new(&config.near.endpoint);

        Ok(NearContract::new(contract_account_id, signer, near_client))
    }

    pub async fn run(&self) -> Result<()> {
        info!("🚀 Starting ETH to NEAR relayer");

        let mut iteration = 0;
        loop {
            iteration += 1;

            if self.should_stop(iteration) {
                info!("Reached maximum iterations ({}), stopping", iteration);
                break;
            }

            info!("=== Relay Loop {} ===", iteration);

            let result = self.run_single_iteration().await;
            let sleep_duration = self.handle_iteration_result(result).await;

            info!(
                "Sleeping for {} seconds before next iteration",
                sleep_duration
            );
            sleep(Duration::from_secs(sleep_duration)).await;
        }

        Ok(())
    }

    fn should_stop(&self, iteration: u64) -> bool {
        self.config
            .relayer
            .max_iterations
            .map(|max| iteration > max)
            .unwrap_or(false)
    }

    async fn run_single_iteration(&self) -> RelayResult {
        // Check synchronization first
        if let Err(e) = self.check_synchronization().await {
            return RelayResult::Error(e);
        }

        // Get client mode and handle accordingly
        match self.get_client_mode().await {
            Ok(ClientMode::SubmitLightClientUpdate) => {
                info!("📡 Light Client Update Mode");
                self.handle_light_client_update().await
            }
            Ok(ClientMode::SubmitHeader) => {
                info!("🔗 Submit Header Mode");
                self.handle_header_submission().await
            }
            Err(e) => RelayResult::Error(e),
        }
    }

    async fn handle_iteration_result(&self, result: RelayResult) -> u64 {
        match result {
            RelayResult::Submitted => {
                info!("✅ Operation completed successfully");
                self.config.relayer.submission_sleep_secs
            }
            RelayResult::NoWorkToDo => {
                info!("⏭️  No work to do at this time");
                self.config.relayer.sync_sleep_secs
            }
            RelayResult::Error(e) => {
                error!("❌ Operation failed: {}", e);
                self.config.relayer.sync_sleep_secs
            }
        }
    }

    async fn check_synchronization(&self) -> Result<()> {
        if self.beacon_client.is_syncing().await? {
            return Err(color_eyre::eyre::eyre!("Beacon node is syncing"));
        }
        info!("✅ All clients are synchronized");
        Ok(())
    }

    async fn get_client_mode(&self) -> Result<ClientMode> {
        self.near_contract
            .get_client_mode()
            .await
            .wrap_err("Failed to get client mode")
    }

    async fn handle_light_client_update(&self) -> RelayResult {
        if self.config.relayer.dry_run {
            info!("🧪 DRY RUN: Would submit light client update");
            return RelayResult::Submitted;
        }

        match self.submit_light_client_update().await {
            Ok(true) => RelayResult::Submitted,
            Ok(false) => RelayResult::NoWorkToDo,
            Err(e) => RelayResult::Error(e),
        }
    }

    async fn handle_header_submission(&self) -> RelayResult {
        if self.config.relayer.dry_run {
            info!("🧪 DRY RUN: Would submit execution headers");
            return RelayResult::Submitted;
        }

        match self.submit_execution_headers().await {
            Ok(true) => RelayResult::Submitted,
            Ok(false) => RelayResult::NoWorkToDo,
            Err(e) => RelayResult::Error(e),
        }
    }

    async fn submit_light_client_update(&self) -> Result<bool> {
        let (near_slot, eth_slot) = self.get_finalized_slots().await?;

        if !self.should_submit_light_client_update(near_slot, eth_slot) {
            return Ok(false);
        }

        let update = self.fetch_appropriate_update(near_slot, eth_slot).await?;
        self.near_contract
            .submit_light_client_update(update)
            .await?;

        Ok(true)
    }

    async fn get_finalized_slots(&self) -> Result<(u64, u64)> {
        let near_slot = self.near_contract.get_finalized_beacon_block_slot().await?;
        let eth_slot = self.beacon_client.get_last_finalized_slot().await?;

        info!("Finalized slots - NEAR: {}, ETH: {}", near_slot, eth_slot);
        Ok((near_slot, eth_slot))
    }

    async fn fetch_appropriate_update(
        &self,
        near_slot: u64,
        eth_slot: u64,
    ) -> Result<eth_types::eth2::LightClientUpdate> {
        let near_period = BeaconLightClientService::get_period_for_slot(near_slot);
        let eth_period = BeaconLightClientService::get_period_for_slot(eth_slot);

        if eth_period == near_period {
            debug!("Periods are the same, fetching finality update");
            self.beacon_client.fetch_finality_update().await
        } else {
            debug!(
                "Periods differ, fetching period update for period {}",
                near_period + 1
            );
            self.beacon_client
                .fetch_period_update(near_period + 1)
                .await
        }
    }

    fn should_submit_light_client_update(&self, near_slot: u64, eth_slot: u64) -> bool {
        if eth_slot <= near_slot {
            debug!(
                "ETH finalized slot ({}) <= NEAR finalized slot ({}), no update needed",
                eth_slot, near_slot
            );
            return false;
        }

        let slot_diff = eth_slot - near_slot;
        let min_slot_diff = 32 * self.config.relayer.update_interval_epochs;

        if slot_diff < min_slot_diff {
            debug!(
                "Slot difference ({}) < minimum required ({}), waiting...",
                slot_diff, min_slot_diff
            );
            return false;
        }

        true
    }

    async fn submit_execution_headers(&self) -> Result<bool> {
        let block_range = self.get_submittable_block_range().await?;

        let Some((start_block, end_block)) = block_range else {
            debug!("No new blocks to submit");
            return Ok(false);
        };

        let headers = self
            .fetch_and_prepare_headers(start_block, end_block)
            .await?;
        self.near_contract
            .submit_execution_headers(&headers)
            .await?;

        Ok(true)
    }

    async fn get_submittable_block_range(&self) -> Result<Option<(u64, u64)>> {
        let last_block_number = self.near_contract.get_last_block_number().await?;
        let max_block_number = self.get_max_submittable_block_number().await?;

        if max_block_number <= last_block_number {
            return Ok(None);
        }

        let start_block = last_block_number + 1;
        let end_block = std::cmp::min(
            max_block_number,
            start_block + self.config.relayer.headers_batch_size as u64 - 1,
        );

        Ok(Some((start_block, end_block)))
    }

    async fn fetch_and_prepare_headers(
        &self,
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<eth_types::BlockHeader>> {
        info!(
            "Fetching execution headers for blocks {} to {}",
            start_block, end_block
        );

        let mut headers = self
            .execution_client
            .fetch_block_range(start_block..=end_block)
            .await?;

        if headers.is_empty() {
            return Err(color_eyre::eyre::eyre!(
                "No headers fetched for range {}..={}",
                start_block,
                end_block
            ));
        }

        info!(
            "Fetched {} headers, preparing for submission",
            headers.len()
        );

        // Reverse for contract submission
        headers.reverse();
        Ok(headers)
    }

    async fn get_max_submittable_block_number(&self) -> Result<u64> {
        if let Some(tail_block) = self
            .near_contract
            .get_unfinalized_tail_block_number()
            .await?
        {
            return Ok(tail_block - 1);
        }

        let finalized_slot = self.near_contract.get_finalized_beacon_block_slot().await?;
        self.beacon_client
            .get_block_number_for_slot(finalized_slot)
            .await
            .wrap_err_with(|| {
                format!(
                    "Failed to get execution block number for finalized beacon slot {}",
                    finalized_slot
                )
            })
    }
}

fn setup_logging(config: &Config) -> Result<()> {
    let level = match config.logging.level.to_lowercase().as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => {
            return Err(color_eyre::eyre::eyre!(
                "Invalid log level: {}",
                config.logging.level
            ));
        }
    };

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false);

    if config.logging.json {
        subscriber.json().init();
    } else {
        subscriber.init();
    }

    Ok(())
}

async fn run_relayer(config: Config) -> Result<()> {
    config
        .validate()
        .wrap_err("Configuration validation failed")?;
    setup_logging(&config).wrap_err("Failed to setup logging")?;
    config.print_summary();

    let relayer = EthRelayer::new(config)
        .await
        .wrap_err("Failed to create relayer")?;
    relayer.run().await.wrap_err("Relayer execution failed")?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Run) {
        Commands::GenerateConfig { output } => {
            let example_config =
                Config::example_toml().wrap_err("Failed to generate example configuration")?;
            std::fs::write(&output, example_config)
                .wrap_err_with(|| format!("Failed to write config to {}", output.display()))?;
            println!("📝 Example configuration written to {}", output.display());
            println!(
                "💡 Edit the file and set your account IDs and secret key path before running."
            );
        }

        Commands::ValidateConfig => {
            let config = Config::load(cli.config).wrap_err("Failed to load configuration")?;
            match config.validate() {
                Ok(()) => {
                    println!("✅ Configuration is valid");
                    config.print_summary();
                }
                Err(e) => {
                    println!("❌ Configuration validation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Run => {
            let config = Config::load(cli.config).wrap_err("Failed to load configuration")?;
            run_relayer(config).await?;
        }
    }

    Ok(())
}
