use color_eyre::Result;

mod common;
use common::TestFixture;

#[tokio::test]
async fn test_contract_deployment() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Verify the contract was deployed successfully
    assert!(fixture.contract.id().as_str().contains("dev-"));
    println!(
        "Contract deployed successfully at: {}",
        fixture.contract.id()
    );

    Ok(())
}

#[tokio::test]
async fn test_contract_deployment_and_initialization() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Verify the contract was deployed successfully
    assert!(fixture.contract.id().as_str().contains("dev-"));

    // Initialize with Sepolia data
    let _init_input = fixture.init_with_sepolia().await?;

    println!(
        "Contract deployed and initialized successfully at: {}",
        fixture.contract.id()
    );
    Ok(())
}

#[tokio::test]
async fn test_contract_initialization_without_validation() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Initialize with validation disabled for faster testing
    let _init_input = fixture.init_with_sepolia_no_validation().await?;

    println!(
        "Contract initialized successfully without validation at: {}",
        fixture.contract.id()
    );
    Ok(())
}

#[tokio::test]
async fn test_eth_light_client_account_id_and_client_getters() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Test the getter methods
    let account_id = fixture.near_client.eth_light_client_account_id();
    let client = fixture.near_client.client();

    assert_eq!(account_id, fixture.contract.id());
    assert_eq!(client.rpc_addr(), fixture.worker.rpc_addr());

    println!("Contract account ID: {}", account_id);
    println!("RPC address: {}", client.rpc_addr());

    Ok(())
}
