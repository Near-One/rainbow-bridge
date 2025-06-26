use color_eyre::Result;
use eth_types::H256;
use eth2_utility::types::ClientMode;

mod common;
use common::{TestFixture, load_test_headers, load_test_light_client_updates};

#[tokio::test]
async fn test_full_lifecycle_smoke_test() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Initialize without validation for faster testing
    let init_input = fixture.init_with_sepolia_no_validation().await?;

    // Get the finalized beacon block hash
    let hash = fixture
        .near_client
        .get_finalized_beacon_block_hash()
        .await?;

    println!("First finalized slot after init: {:?}", hash);

    // Verify initial state
    use tree_hash::TreeHash;
    assert_eq!(
        hash,
        init_input
            .finalized_beacon_header
            .header
            .tree_hash_root()
            .0
            .into()
    );

    // Load test data
    let (_, mut first_update) = load_test_light_client_updates()?;
    let headers = load_test_headers()?;

    // Pick first 32 blocks from our window
    let slice = &headers[1..33];
    let last_block_hash = slice.last().unwrap().calculate_hash();
    first_update.finalized_header.execution.block_hash = last_block_hash;

    // Reverse the order of slice for submission
    let reversed: Vec<_> = slice.iter().rev().cloned().collect();

    println!("Submitting light client update to enable SubmitHeader mode...");
    fixture
        .near_client
        .submit_light_client_update(first_update.clone())
        .await?;

    // Check the mode changed
    let mode = fixture.near_client.get_client_mode().await?;
    assert_eq!(mode, ClientMode::SubmitHeader);

    println!("Submitting execution headers...");
    fixture
        .near_client
        .submit_execution_headers(&reversed)
        .await?;

    // Verify that each block's hash is stored
    for header in slice {
        let result: Option<H256> = fixture.near_client.get_block_hash(header.number).await?;
        assert!(result.is_some(), "block {} missing", header.number);

        // Verify the stored hash matches the calculated hash
        let stored_hash = result.unwrap();
        let calculated_hash = header.calculate_hash();
        assert_eq!(
            stored_hash, calculated_hash,
            "Hash mismatch for block {}: stored {:?} vs calculated {:?}",
            header.number, stored_hash, calculated_hash
        );
    }

    println!("✅ Full lifecycle test completed successfully");
    Ok(())
}

#[tokio::test]
async fn test_light_client_update_submission() -> Result<()> {
    let fixture = TestFixture::new().await?;
    fixture.init_with_sepolia_no_validation().await?;

    // Load test updates
    let (_, first_update) = load_test_light_client_updates()?;

    // Verify initial mode
    let initial_mode = fixture.near_client.get_client_mode().await?;
    assert_eq!(initial_mode, ClientMode::SubmitLightClientUpdate);

    // Submit light client update
    fixture
        .near_client
        .submit_light_client_update(first_update)
        .await?;

    // Verify mode changed
    let new_mode = fixture.near_client.get_client_mode().await?;
    assert_eq!(new_mode, ClientMode::SubmitHeader);

    println!("✅ Light client update submission test passed");
    Ok(())
}

#[tokio::test]
async fn test_execution_headers_submission() -> Result<()> {
    let fixture = TestFixture::new().await?;
    fixture.init_with_sepolia_no_validation().await?;

    // First submit a light client update to get into SubmitHeader mode
    let (_, first_update) = load_test_light_client_updates()?;
    let headers = load_test_headers()?;

    // Modify the update to match our test headers
    let mut modified_update = first_update;
    let test_slice = &headers[1..5]; // Use just a few headers for this test
    let last_block_hash = test_slice.last().unwrap().calculate_hash();
    modified_update.finalized_header.execution.block_hash = last_block_hash;

    fixture
        .near_client
        .submit_light_client_update(modified_update)
        .await?;

    // Verify we're in SubmitHeader mode
    let mode = fixture.near_client.get_client_mode().await?;
    assert_eq!(mode, ClientMode::SubmitHeader);

    // Submit execution headers (reversed order)
    let reversed_headers: Vec<_> = test_slice.iter().rev().cloned().collect();
    fixture
        .near_client
        .submit_execution_headers(&reversed_headers)
        .await?;

    // Verify headers were stored
    for header in test_slice {
        let result = fixture.near_client.get_block_hash(header.number).await?;
        assert!(
            result.is_some(),
            "Header {} should be stored",
            header.number
        );
    }

    println!("✅ Execution headers submission test passed");
    Ok(())
}

#[tokio::test]
async fn test_empty_headers_submission() -> Result<()> {
    let fixture = TestFixture::new().await?;
    fixture.init_with_sepolia_no_validation().await?;

    // Try to submit empty headers array - should not fail
    let empty_headers: Vec<eth_types::BlockHeader> = vec![];
    let result = fixture
        .near_client
        .submit_execution_headers(&empty_headers)
        .await;

    // Should succeed (no-op)
    assert!(result.is_ok(), "Empty headers submission should succeed");

    println!("✅ Empty headers submission test passed");
    Ok(())
}
