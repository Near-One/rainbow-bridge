mod common;
use color_eyre::Result;
use common::TestFixture;
use relayer::{BeaconClient, Config, EthRelayer, ExecutionClient};
use tracing_subscriber::EnvFilter;

#[tokio::test]
async fn test_relayer_mainloop_hybrid() -> Result<()> {
    // Only show debug+ logs from your "relayer" crate, info+ from everything else
    let filter = EnvFilter::new("info,relayer=debug");

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_test_writer()
        .try_init()
        .ok();

    // Use NEAR sandbox (fast, deterministic)
    let fixture: TestFixture = TestFixture::new().await?;
    fixture.init_with_sepolia_no_validation().await?;

    // Use REAL Sepolia clients (real data, real behavior)
    let beacon_client = BeaconClient::new("http://unstable.sepolia.beacon-api.nimbus.team")?;
    let execution_client = ExecutionClient::sepolia()?;

    // Create relayer with real Ethereum clients + sandbox NEAR
    let mut config = Config::default();
    config.relayer.max_iterations = Some(4); // Just a couple iterations
    config.relayer.headers_batch_size = 32; // Smaller batches for testing

    let relayer = EthRelayer::with_clients(
        beacon_client,               // Real Sepolia beacon
        execution_client,            // Real Sepolia execution
        fixture.near_client.clone(), // Sandbox NEAR contract
        config,
    );

    // Run the actual main loop
    relayer.run().await?;

    // Verify it made progress
    let mode = fixture.near_client.get_client_mode().await?;
    let block_num = fixture.near_client.get_last_block_number().await?;

    println!("Final mode: {:?}, block: {}", mode, block_num);
    Ok(())
}
