use color_eyre::Result;
use color_eyre::eyre::Context;
use tree_hash::TreeHash;

mod common;
use common::TestFixture;

#[tokio::test]
async fn test_get_finalized_beacon_block_hash() -> Result<()> {
    let fixture = TestFixture::new().await?;
    let init_input = fixture.init_with_sepolia().await?;

    // Test the view call
    let hash = fixture
        .near_client
        .get_finalized_beacon_block_hash()
        .await
        .wrap_err("Failed to get finalized beacon block hash")?;

    println!("Finalized beacon block hash: {:?}", hash);

    // Verify it matches the expected hash from initialization
    assert_eq!(
        hash,
        init_input
            .finalized_beacon_header
            .beacon
            .tree_hash_root()
            .0
            .into()
    );

    Ok(())
}

#[tokio::test]
async fn test_get_finalized_beacon_block_slot() -> Result<()> {
    let fixture = TestFixture::new().await?;
    fixture.init_with_sepolia().await?;

    let slot = fixture
        .near_client
        .get_finalized_beacon_block_slot()
        .await
        .wrap_err("Failed to get finalized beacon block slot")?;

    println!("Finalized beacon block slot: {}", slot);
    // Verify it's a reasonable slot number
    assert!(slot > 0, "Slot number should be greater than 0");

    Ok(())
}

#[tokio::test]
async fn test_get_client_mode() -> Result<()> {
    let fixture = TestFixture::new().await?;
    fixture.init_with_sepolia().await?;

    let mode = fixture
        .near_client
        .get_client_mode()
        .await
        .wrap_err("Failed to get client mode")?;

    println!("Client mode: {:?}", mode);
    // After initialization, should be in SubmitLightClientUpdate mode
    use eth2_utility::types::ClientMode;
    assert_eq!(mode, ClientMode::SubmitLightClientUpdate);

    Ok(())
}

#[tokio::test]
async fn test_get_light_client_state() -> Result<()> {
    let fixture = TestFixture::new().await?;
    fixture.init_with_sepolia().await?;

    let _state = fixture
        .near_client
        .get_light_client_state()
        .await
        .wrap_err("Failed to get light client state")?;

    println!("Light client state retrieved successfully");
    Ok(())
}

#[tokio::test]
async fn test_get_last_block_number() -> Result<()> {
    let fixture = TestFixture::new().await?;
    fixture.init_with_sepolia().await?;

    let block_number = fixture
        .near_client
        .get_last_block_number()
        .await
        .wrap_err("Failed to get last block number")?;

    println!("Last block number: {}", block_number);
    assert!(block_number > 0, "Block number should be greater than 0");

    Ok(())
}

#[tokio::test]
async fn test_get_unfinalized_tail_block_number() -> Result<()> {
    let fixture = TestFixture::new().await?;
    fixture.init_with_sepolia().await?;

    let block_number_opt = fixture
        .near_client
        .get_unfinalized_tail_block_number()
        .await
        .wrap_err("Failed to get unfinalized tail block number")?;

    println!("Unfinalized tail block number: {:?}", block_number_opt);
    // This returns an Option<u64>, so None is valid
    if let Some(block_number) = block_number_opt {
        assert!(
            block_number > 0,
            "Block number should be greater than 0 when present"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_get_block_hash() -> Result<()> {
    let fixture = TestFixture::new().await?;
    let init_input = fixture.init_with_sepolia().await?;

    // Test getting the hash for the initialized block
    let block_number = init_input.finalized_execution_header.number;
    let result = fixture.near_client.get_block_hash(block_number).await?;

    println!("Block hash for block {}: {:?}", block_number, result);
    assert!(result.is_some(), "Should have hash for initialized block");

    // Test getting hash for non-existent block
    let non_existent_block = block_number + 1000000;
    let result = fixture
        .near_client
        .get_block_hash(non_existent_block)
        .await?;

    assert!(
        result.is_none(),
        "Should return None for non-existent block"
    );

    Ok(())
}

#[tokio::test]
async fn test_all_view_methods_sequentially() -> Result<()> {
    let fixture = TestFixture::new().await?;
    fixture.init_with_sepolia().await?;

    println!("Testing all view methods sequentially...");

    // Test all view methods in sequence - all should succeed
    let _hash = fixture
        .near_client
        .get_finalized_beacon_block_hash()
        .await
        .wrap_err("Failed to get finalized beacon block hash")?;

    let slot = fixture
        .near_client
        .get_finalized_beacon_block_slot()
        .await
        .wrap_err("Failed to get finalized beacon block slot")?;

    let _mode = fixture
        .near_client
        .get_client_mode()
        .await
        .wrap_err("Failed to get client mode")?;

    let _state = fixture
        .near_client
        .get_light_client_state()
        .await
        .wrap_err("Failed to get light client state")?;

    let block_number = fixture
        .near_client
        .get_last_block_number()
        .await
        .wrap_err("Failed to get last block number")?;

    let _tail_block = fixture
        .near_client
        .get_unfinalized_tail_block_number()
        .await
        .wrap_err("Failed to get unfinalized tail block number")?;

    // Basic sanity checks
    assert!(slot > 0, "Slot should be greater than 0");
    assert!(block_number > 0, "Block number should be greater than 0");

    println!("All view methods called and validated successfully");
    Ok(())
}
