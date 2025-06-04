#[cfg(test)]
mod integration_tests {

    use color_eyre::Result;
    use color_eyre::eyre::Context;
    use eth_types::eth2::LightClientUpdate;
    use eth_types::{BlockHeader, H256};
    use eth2_utility::consensus::Network;
    use eth2_utility::types::{ClientMode, InitInput};
    use near_crypto::{InMemorySigner, SecretKey};
    use near_workspaces::network::Sandbox;
    use near_workspaces::{Contract, Worker};
    use relayer::near::NearContract;
    use tree_hash::TreeHash;
    /// Simple helper to load Sepolia test data
    fn load_sepolia_init_data() -> Result<InitInput> {
        // Read the initial sync committee (period 925)
        let init_update: LightClientUpdate = serde_json::from_reader(std::fs::File::open(
            "./tests/data/light_client_update_period_925.json",
        )?)?;

        // Read the first update (period 926)
        let first_update: LightClientUpdate = serde_json::from_reader(std::fs::File::open(
            "./tests/data/light_client_update_period_926.json",
        )?)?;

        // Read the execution headers
        let headers: Vec<BlockHeader> = serde_json::from_reader(std::fs::File::open(
            "./tests/data/execution_blocks_8286935_8295112.json",
        )?)?;

        let init_input = InitInput {
            network: Network::Sepolia,
            finalized_execution_header: headers[0].clone(),
            finalized_beacon_header: first_update.finalized_header.clone(),
            current_sync_committee: init_update
                .next_sync_committee
                .ok_or_else(|| color_eyre::eyre::eyre!("Missing sync committee in init update"))?,
            next_sync_committee: first_update
                .next_sync_committee
                .ok_or_else(|| color_eyre::eyre::eyre!("Missing sync committee in first update"))?,
            validate_updates: true,
            verify_bls_signatures: true,
            hashes_gc_threshold: 51_000,
            trusted_signer: None,
        };

        Ok(init_input)
    }

    /// Test fixture that sets up the sandbox environment and deploys the contract
    struct TestFixture {
        worker: Worker<Sandbox>,
        contract: Contract,
        near_contract: NearContract,
    }

    impl TestFixture {
        async fn new() -> Result<Self> {
            // Install color-eyre, ignoring error if already installed
            let _ = color_eyre::install();

            // Compile the eth2-client
            let wasm = near_workspaces::compile_project("../contracts/near/eth2-client")
                .await
                .wrap_err("Failed to compile eth2-client contract")?;

            // Create sandbox environment
            let worker = near_workspaces::sandbox()
                .await
                .wrap_err("Failed to create sandbox environment")?;

            // Deploy the contract
            let contract = worker
                .dev_deploy(&wasm)
                .await
                .wrap_err("Failed to deploy contract")?;

            let alice = worker
                .dev_create_account()
                .await
                .wrap_err("Failed to create test account")?;

            let secret_key: SecretKey = alice
                .secret_key()
                .to_string()
                .parse()
                .wrap_err("Failed to parse secret key")?;

            let signer = InMemorySigner::from_secret_key(alice.id().clone(), secret_key.clone());

            // Create the near-fetch client pointing to the sandbox RPC
            let near_fetch_client = near_fetch::Client::new(&worker.rpc_addr());

            // Create our NearContract wrapper
            let near_contract = NearContract::new(contract.id().clone(), signer, near_fetch_client);

            Ok(Self {
                worker,
                contract,
                near_contract,
            })
        }
    }

    #[tokio::test]
    async fn test_contract_deployment_and_initialization() -> Result<()> {
        let fixture = TestFixture::new().await?;

        // Verify the contract was deployed successfully
        assert!(fixture.contract.id().as_str().contains("dev-"));

        let init_input = load_sepolia_init_data().wrap_err("Failed to load Sepolia test data")?;

        fixture
            .near_contract
            .init_contract(init_input)
            .await
            .wrap_err("Failed to initialize contract")?;

        println!(
            "Contract deployed and initialized successfully at: {}",
            fixture.contract.id()
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_full_lifecycle_smoke_test() -> Result<()> {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        let fixture = TestFixture::new().await?;
        let mut init_input =
            load_sepolia_init_data().wrap_err("Failed to load Sepolia test data")?;
        init_input.validate_updates = false;
        init_input.verify_bls_signatures = false;
        fixture
            .near_contract
            .init_contract(init_input.clone())
            .await?;

        // Get the finalized beacon block hash
        let hash = fixture
            .near_contract
            .get_finalized_beacon_block_hash()
            .await
            .wrap_err("Failed to get finalized beacon block hash")?;

        println!("First finalized slot after init: {:?}", hash);

        assert_eq!(
            hash,
            init_input
                .clone()
                .finalized_beacon_header
                .beacon
                .tree_hash_root()
                .0
                .into()
        );

        let mut first_update: LightClientUpdate = serde_json::from_reader(std::fs::File::open(
            "./tests/data/light_client_update_period_926.json",
        )?)?;
        let headers: Vec<BlockHeader> = serde_json::from_reader(std::fs::File::open(
            "./tests/data/execution_blocks_8286935_8295112.json",
        )?)?;

        // pick first 32 blocks from our window
        let slice = &headers[1..33];
        let last_block_hash = slice.last().unwrap().calculate_hash();
        first_update.finalized_header.execution.block_hash = last_block_hash;

        //Reverse the order of slice
        let reversed: Vec<BlockHeader> = slice.iter().rev().cloned().collect();

        println!("Submitting light client update to enable SubmitHeader mode...");
        fixture
            .near_contract
            .submit_light_client_update(first_update.clone())
            .await
            .wrap_err("Failed to submit light client update")?;

        // Check the mode
        let mode = fixture.near_contract.get_client_mode().await?;
        assert!(mode == ClientMode::SubmitHeader);

        fixture
            .near_contract
            .submit_execution_headers(&reversed)
            .await
            .wrap_err("Failed to submit execution headers")?;

        // Verify that each block’s hash is stored
        for header in slice {
            let result: Option<H256> = fixture.near_contract.get_block_hash(header.number).await?;
            assert!(result.is_some(), "block {} missing", header.number);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_get_finalized_beacon_block_hash() -> Result<()> {
        let fixture = TestFixture::new().await?;

        let init_input = load_sepolia_init_data()?;
        fixture.near_contract.init_contract(init_input).await?;

        // Test the view call
        let hash = fixture
            .near_contract
            .get_finalized_beacon_block_hash()
            .await
            .wrap_err("Failed to get finalized beacon block hash")?;

        println!("Finalized beacon block hash: {:?}", hash);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_finalized_beacon_block_slot() -> Result<()> {
        let fixture = TestFixture::new().await?;

        let init_input = load_sepolia_init_data()?;
        fixture.near_contract.init_contract(init_input).await?;

        let slot = fixture
            .near_contract
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

        let init_input = load_sepolia_init_data()?;
        fixture.near_contract.init_contract(init_input).await?;

        let mode = fixture
            .near_contract
            .get_client_mode()
            .await
            .wrap_err("Failed to get client mode")?;

        println!("Client mode: {:?}", mode);
        // Add specific assertions based on your ClientMode enum as needed

        Ok(())
    }

    #[tokio::test]
    async fn test_get_light_client_state() -> Result<()> {
        let fixture = TestFixture::new().await?;

        let init_input = load_sepolia_init_data()?;
        fixture.near_contract.init_contract(init_input).await?;

        let _state = fixture
            .near_contract
            .get_light_client_state()
            .await
            .wrap_err("Failed to get light client state")?;

        println!("Light client state retrieved successfully");
        // Add specific assertions based on your LightClientState structure as needed

        Ok(())
    }

    #[tokio::test]
    async fn test_get_last_block_number() -> Result<()> {
        let fixture = TestFixture::new().await?;

        let init_input = load_sepolia_init_data()?;
        fixture.near_contract.init_contract(init_input).await?;

        let block_number = fixture
            .near_contract
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

        let init_input = load_sepolia_init_data()?;
        fixture.near_contract.init_contract(init_input).await?;

        let block_number_opt = fixture
            .near_contract
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
    async fn test_all_view_methods_sequentially() -> Result<()> {
        let fixture = TestFixture::new().await?;

        let init_input = load_sepolia_init_data()?;
        fixture.near_contract.init_contract(init_input).await?;

        println!("Testing all view methods sequentially...");

        // Test all view methods in sequence - all should succeed
        let _hash = fixture
            .near_contract
            .get_finalized_beacon_block_hash()
            .await
            .wrap_err("Failed to get finalized beacon block hash")?;

        let slot = fixture
            .near_contract
            .get_finalized_beacon_block_slot()
            .await
            .wrap_err("Failed to get finalized beacon block slot")?;

        let _mode = fixture
            .near_contract
            .get_client_mode()
            .await
            .wrap_err("Failed to get client mode")?;

        let _state = fixture
            .near_contract
            .get_light_client_state()
            .await
            .wrap_err("Failed to get light client state")?;

        let block_number = fixture
            .near_contract
            .get_last_block_number()
            .await
            .wrap_err("Failed to get last block number")?;

        let _tail_block = fixture
            .near_contract
            .get_unfinalized_tail_block_number()
            .await
            .wrap_err("Failed to get unfinalized tail block number")?;

        // Basic sanity checks
        assert!(slot > 0, "Slot should be greater than 0");
        assert!(block_number > 0, "Block number should be greater than 0");

        println!("All view methods called and validated successfully");
        Ok(())
    }

    #[tokio::test]
    async fn test_contract_account_id_and_client_getters() -> Result<()> {
        let fixture = TestFixture::new().await?;

        // Test the getter methods
        let account_id = fixture.near_contract.contract_account_id();
        let client = fixture.near_contract.client();

        assert_eq!(account_id, fixture.contract.id());
        assert_eq!(client.rpc_addr(), fixture.worker.rpc_addr());

        println!("Contract account ID: {}", account_id);
        println!("RPC address: {}", client.rpc_addr());

        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling_uninitialized_contract() -> Result<()> {
        let fixture = TestFixture::new().await?;

        // Test calling methods on uninitialized contract should return errors
        let result = fixture
            .near_contract
            .get_finalized_beacon_block_hash()
            .await;

        assert!(
            result.is_err(),
            "Should fail when contract is not initialized"
        );

        Ok(())
    }
}
