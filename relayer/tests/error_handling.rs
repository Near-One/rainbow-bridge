use color_eyre::Result;

mod common;
use common::TestFixture;

#[tokio::test]
async fn test_error_handling_uninitialized_contract() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Test calling methods on uninitialized contract should return errors
    let result = fixture.near_client.get_finalized_beacon_block_hash().await;

    assert!(
        result.is_err(),
        "Should fail when contract is not initialized"
    );

    println!("✅ Uninitialized contract error handling works correctly");
    Ok(())
}

#[tokio::test]
async fn test_view_methods_fail_on_uninitialized_contract() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Test get_finalized_beacon_block_hash
    let result = fixture.near_client.get_finalized_beacon_block_hash().await;
    assert!(
        result.is_err(),
        "get_finalized_beacon_block_hash should fail on uninitialized contract"
    );
    println!("✅ get_finalized_beacon_block_hash correctly fails on uninitialized contract");

    // Test get_finalized_beacon_block_slot
    let result = fixture.near_client.get_finalized_beacon_block_slot().await;
    assert!(
        result.is_err(),
        "get_finalized_beacon_block_slot should fail on uninitialized contract"
    );
    println!("✅ get_finalized_beacon_block_slot correctly fails on uninitialized contract");

    // Test get_client_mode
    let result = fixture.near_client.get_client_mode().await;
    assert!(
        result.is_err(),
        "get_client_mode should fail on uninitialized contract"
    );
    println!("✅ get_client_mode correctly fails on uninitialized contract");

    // Test get_light_client_state
    let result = fixture.near_client.get_light_client_state().await;
    assert!(
        result.is_err(),
        "get_light_client_state should fail on uninitialized contract"
    );
    println!("✅ get_light_client_state correctly fails on uninitialized contract");

    // Test get_last_block_number
    let result = fixture.near_client.get_last_block_number().await;
    assert!(
        result.is_err(),
        "get_last_block_number should fail on uninitialized contract"
    );
    println!("✅ get_last_block_number correctly fails on uninitialized contract");

    // Test get_unfinalized_tail_block_number
    let result = fixture
        .near_client
        .get_unfinalized_tail_block_number()
        .await;
    assert!(
        result.is_err(),
        "get_unfinalized_tail_block_number should fail on uninitialized contract"
    );
    println!("✅ get_unfinalized_tail_block_number correctly fails on uninitialized contract");

    Ok(())
}

#[tokio::test]
async fn test_get_block_hash_nonexistent_block() -> Result<()> {
    let fixture = TestFixture::new().await?;
    fixture.init_with_sepolia().await?;

    // Test getting hash for a very high block number that definitely doesn't exist
    let non_existent_block = u64::MAX;
    let result = fixture
        .near_client
        .get_block_hash(non_existent_block)
        .await?;

    assert!(
        result.is_none(),
        "Should return None for non-existent block {}",
        non_existent_block
    );

    // Test with block number 0 (shouldn't exist in our test data)
    let result = fixture.near_client.get_block_hash(0).await?;

    assert!(result.is_none(), "Should return None for block 0");

    println!("✅ Non-existent block handling works correctly");
    Ok(())
}

#[tokio::test]
async fn test_contract_resilience() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Test that multiple contract creations work
    let fixture2 = TestFixture::new().await?;

    // Both should have different contract IDs
    assert_ne!(
        fixture.contract.id(),
        fixture2.contract.id(),
        "Different contract instances should have different IDs"
    );

    println!(
        "Contract 1: {}, Contract 2: {}",
        fixture.contract.id(),
        fixture2.contract.id()
    );

    println!("✅ Contract resilience test passed");
    Ok(())
}
