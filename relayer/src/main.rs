use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use color_eyre::{Result, eyre::Context};
use eth2_utility::types::ClientMode;
use near_crypto::{InMemorySigner, SecretKey};
use relayer::{beacon::BeaconLightClientService, execution::ExecutionClient, near::NearContract};
use tokio::time::sleep;
use tracing::{debug, error, info, level_filters::LevelFilter, warn};

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

        // Load NEAR signer secret key
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

        // Create NEAR signer and client
        let signer = InMemorySigner::from_secret_key(signer_account_id, secret_key);
        let near_client = near_fetch::Client::new(&config.near.endpoint);
        let near_contract = NearContract::new(contract_account_id, signer, near_client);

        Ok(Self {
            beacon_client,
            execution_client,
            near_contract,
            config,
        })
    }

    pub async fn run(&self) -> Result<()> {
        info!("🚀 Starting ETH to NEAR relayer");

        let mut iteration = 0;

        loop {
            iteration += 1;

            if let Some(max_iter) = self.config.relayer.max_iterations {
                if iteration > max_iter {
                    info!("Reached maximum iterations ({}), stopping", max_iter);
                    break;
                }
            }

            info!("=== Relay Loop {} ===", iteration);

            // Wait for synchronization
            if let Err(e) = self.wait_for_synchronization().await {
                warn!("Synchronization check failed: {}, sleeping...", e);
                sleep(Duration::from_secs(self.config.relayer.sync_sleep_secs)).await;
                continue;
            }

            // Get the current client mode from NEAR contract
            let client_mode = match self.near_contract.get_client_mode().await {
                Ok(mode) => mode,
                Err(e) => {
                    warn!("Failed to get client mode: {}, sleeping...", e);
                    sleep(Duration::from_secs(self.config.relayer.sync_sleep_secs)).await;
                    continue;
                }
            };

            debug!("Current client mode: {:?}", client_mode);

            let submitted_in_iteration = match client_mode {
                ClientMode::SubmitLightClientUpdate => self.handle_light_client_update_mode().await,
                ClientMode::SubmitHeader => self.handle_submit_header_mode().await,
            };

            // Sleep between iterations
            let sleep_duration = if submitted_in_iteration {
                self.config.relayer.submission_sleep_secs
            } else {
                self.config.relayer.sync_sleep_secs
            };

            info!(
                "Sleeping for {} seconds before next iteration",
                sleep_duration
            );
            sleep(Duration::from_secs(sleep_duration)).await;
        }

        Ok(())
    }

    async fn wait_for_synchronization(&self) -> Result<()> {
        if self.beacon_client.is_syncing().await? {
            info!("⏳ Beacon node is syncing, waiting...");
            return Err(color_eyre::eyre::eyre!("Beacon node is syncing"));
        }

        info!("✅ All clients are synchronized");
        Ok(())
    }

    async fn handle_light_client_update_mode(&self) -> bool {
        info!("📡 Light Client Update Mode");

        if self.config.relayer.dry_run {
            info!("🧪 DRY RUN: Would submit light client update");
            return true;
        }

        match self.submit_light_client_update().await {
            Ok(submitted) => {
                if submitted {
                    info!("✅ Light client update submitted successfully");
                } else {
                    info!("⏭️  No light client update needed at this time");
                }
                submitted
            }
            Err(e) => {
                error!("❌ Failed to handle light client update: {}", e);
                false
            }
        }
    }

    async fn handle_submit_header_mode(&self) -> bool {
        info!("🔗 Submit Header Mode");

        if self.config.relayer.dry_run {
            info!("🧪 DRY RUN: Would submit execution headers");
            return true;
        }

        match self.submit_execution_headers().await {
            Ok(submitted) => {
                if submitted {
                    info!("✅ Execution headers submitted successfully");
                } else {
                    info!("⏭️  No headers to submit at this time");
                }
                submitted
            }
            Err(e) => {
                error!("❌ Failed to handle header submission: {}", e);
                false
            }
        }
    }

    async fn submit_light_client_update(&self) -> Result<bool> {
        // Get finalized slots from both chains
        let near_finalized_slot = self.near_contract.get_finalized_beacon_block_slot().await?;
        let eth_finalized_slot = self.beacon_client.get_last_finalized_slot().await?;

        info!(
            "Finalized slots - NEAR: {}, ETH: {}",
            near_finalized_slot, eth_finalized_slot
        );

        // Check if we need to submit an update
        if !self.should_submit_light_client_update(near_finalized_slot, eth_finalized_slot) {
            return Ok(false);
        }

        // Determine which type of update to submit
        let near_period = BeaconLightClientService::get_period_for_slot(near_finalized_slot);
        let eth_period = BeaconLightClientService::get_period_for_slot(eth_finalized_slot);

        let update = if eth_period == near_period {
            // Same period - submit finality update
            debug!("Periods are the same, fetching finality update");
            self.beacon_client.fetch_finality_update().await?
        } else {
            // Different periods - submit period update with sync committee
            debug!(
                "Periods differ, fetching period update for period {}",
                near_period + 1
            );
            self.beacon_client
                .fetch_period_update(near_period + 1)
                .await?
        };

        // Submit the update
        self.near_contract
            .submit_light_client_update(update)
            .await?;

        Ok(true)
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
        let min_slot_diff = 32 * self.config.relayer.update_interval_epochs; // 32 slots per epoch

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
        // Get the range of blocks to submit
        let last_block_number = self.near_contract.get_last_block_number().await?;
        let max_block_number = self.get_max_submittable_block_number().await?;

        if max_block_number <= last_block_number {
            debug!(
                "No new blocks to submit (last: {}, max: {})",
                last_block_number, max_block_number
            );
            return Ok(false);
        }

        let start_block = last_block_number + 1;
        let end_block = std::cmp::min(
            max_block_number,
            start_block + self.config.relayer.headers_batch_size as u64 - 1,
        );

        info!(
            "Fetching execution headers for blocks {} to {}",
            start_block, end_block
        );

        // Fetch the block headers
        let headers = self
            .execution_client
            .fetch_block_range(start_block..=end_block)
            .await?;

        if headers.is_empty() {
            warn!(
                "No headers fetched for range {}..={}",
                start_block, end_block
            );
            return Ok(false);
        }

        info!("Fetched {} headers, submitting to NEAR", headers.len());

        // Submit headers (they need to be in reverse order for the contract)
        let mut reversed_headers = headers;
        reversed_headers.reverse();

        self.near_contract
            .submit_execution_headers(&reversed_headers)
            .await?;

        Ok(true)
    }

    async fn get_max_submittable_block_number(&self) -> Result<u64> {
        // Check if there's an unfinalized tail block number
        if let Some(tail_block) = self
            .near_contract
            .get_unfinalized_tail_block_number()
            .await?
        {
            Ok(tail_block - 1)
        } else {
            // No unfinalized blocks, use the finalized beacon block's execution block
            let finalized_slot = self.near_contract.get_finalized_beacon_block_slot().await?;

            // Get the execution block number corresponding to the finalized slot
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
    // Validate configuration
    config
        .validate()
        .wrap_err("Configuration validation failed")?;

    // Setup logging
    setup_logging(&config).wrap_err("Failed to setup logging")?;

    // Print configuration summary
    config.print_summary();

    // Create and run the relayer
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
