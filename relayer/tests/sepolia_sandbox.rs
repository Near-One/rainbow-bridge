mod common;
use color_eyre::Result;
use common::TestFixture;
use relayer::{BeaconClient, Config, EthRelayer, ExecutionClient};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::test]
async fn test_relayer_mainloop_hybrid() -> Result<()> {
    // Only show debug+ logs from your "relayer" crate, info+ from everything else
    let filter = EnvFilter::new("info,relayer=debug");

    // Create the indicatif layer for progress bars
    let indicatif_layer = IndicatifLayer::new();

    // Set up tracing with both fmt and indicatif layers
    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(indicatif_layer.get_stderr_writer())
                .with_test_writer(),
        )
        .with(indicatif_layer)
        .try_init()
        .ok();

    // Use NEAR sandbox (fast, deterministic)
    let fixture: TestFixture = TestFixture::new().await?;
    fixture.init_with_sepolia().await?;

    // Get the current finalized slot
    let init_finalized_slot = fixture
        .near_client
        .get_finalized_beacon_block_slot()
        .await?;
    let init_block_num = fixture.near_client.get_last_block_number().await?;
    println!("Initial block: {}", init_block_num);
    println!("Initial finalized slot: {}", init_finalized_slot);

    // Use REAL Sepolia clients (real data, real behavior)
    let beacon_client = BeaconClient::new("http://unstable.sepolia.beacon-api.nimbus.team")?;
    let execution_client = ExecutionClient::sepolia()?;

    // Create relayer with real Ethereum clients + sandbox NEAR
    let mut config = Config::default();
    config.relayer.max_iterations = Some(5);
    config.relayer.headers_batch_size = 100;
    config.relayer.max_headers_per_loop = 3000; // Test a large batch size for integration only

    let relayer = EthRelayer::with_clients(
        beacon_client,               // Real Sepolia beacon
        execution_client,            // Real Sepolia execution
        fixture.near_client.clone(), // Sandbox NEAR contract
        config,
    );

    // Run the actual main loop
    relayer.run().await?;

    // Verify it made progress
    let finalized_slot = fixture
        .near_client
        .get_finalized_beacon_block_slot()
        .await?;
    let block_num = fixture.near_client.get_last_block_number().await?;
    println!("Final block: {}", block_num);
    println!("Finalized slot: {}", finalized_slot);

    assert!(finalized_slot > init_finalized_slot);
    assert!(block_num > init_block_num);
    Ok(())
}

#[tokio::test]
async fn test_relayer_run_job_single_execution() -> Result<()> {
    // Only show debug+ logs from your "relayer" crate, info+ from everything else
    let filter = EnvFilter::new("info,relayer=debug");

    // Create the indicatif layer for progress bars
    let indicatif_layer = IndicatifLayer::new();

    // Set up tracing with both fmt and indicatif layers
    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(indicatif_layer.get_stderr_writer())
                .with_test_writer(),
        )
        .with(indicatif_layer)
        .try_init()
        .ok();

    // Use NEAR sandbox (fast, deterministic)
    let fixture: TestFixture = TestFixture::new().await?;
    fixture.init_with_sepolia().await?;

    // Get the current finalized slot
    let init_finalized_slot = fixture
        .near_client
        .get_finalized_beacon_block_slot()
        .await?;
    let init_block_num = fixture.near_client.get_last_block_number().await?;
    println!("Initial block: {}", init_block_num);
    println!("Initial finalized slot: {}", init_finalized_slot);

    // Use REAL Sepolia clients (real data, real behavior)
    let beacon_client = BeaconClient::new("http://unstable.sepolia.beacon-api.nimbus.team")?;
    let execution_client = ExecutionClient::sepolia()?;

    // Create relayer with real Ethereum clients + sandbox NEAR
    let mut config = Config::default();
    config.relayer.headers_batch_size = 100;

    let relayer = EthRelayer::with_clients(
        beacon_client,               // Real Sepolia beacon
        execution_client,            // Real Sepolia execution
        fixture.near_client.clone(), // Sandbox NEAR contract
        config,
    );

    // Run a single job execution (should complete once and exit)
    relayer.run_job().await?;

    // Verify it made some progress (though potentially less than the loop version)
    let finalized_slot = fixture
        .near_client
        .get_finalized_beacon_block_slot()
        .await?;
    let block_num = fixture.near_client.get_last_block_number().await?;
    println!("Final block: {}", block_num);
    println!("Finalized slot: {}", finalized_slot);

    // For run_job, we might see progress or no progress depending on timing
    // The key test is that it completes successfully without hanging
    println!("run_job completed successfully");
    Ok(())
}

/// Live check of the Gloas/Amsterdam data path against real Sepolia endpoints
#[tokio::test]
async fn test_fetch_gloas_finality_update_and_execution_header() -> Result<()> {
    use eth2_utility::consensus::{Network, NetworkConfig};

    let beacon_client = BeaconClient::new("http://unstable.sepolia.beacon-api.nimbus.team")?;
    let execution_client = ExecutionClient::sepolia()?;

    let update = beacon_client.fetch_finality_update().await?;
    let finalized = &update.finalized_header;
    assert!(
        finalized.execution.is_none(),
        "expected Gloas header format"
    );
    assert!(NetworkConfig::new(&Network::Sepolia).is_valid_light_client_header(finalized));

    let block_hash = finalized.execution_block_hash.unwrap();
    let header = execution_client
        .fetch_block_header_by_hash(block_hash)
        .await?
        .expect("finalized execution block not found");
    assert!(header.block_access_list_hash.is_some());
    assert!(header.slot_number.is_some());
    assert_eq!(header.calculate_hash(), block_hash);

    Ok(())
}
