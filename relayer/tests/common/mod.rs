#![allow(dead_code)] // Allow unused functions in test utilities

use color_eyre::Result;
use color_eyre::eyre::Context;
use eth_types::BlockHeader;
use eth_types::eth2::LightClientUpdate;
use eth2_utility::consensus::Network;
use eth2_utility::types::InitInput;
use near_crypto::{InMemorySigner, SecretKey};
use near_workspaces::network::Sandbox;
use near_workspaces::{Contract, Worker};
use relayer::{ContractClient, config::RelayerConfig};

/// Test fixture that sets up the sandbox environment and deploys the contract
pub struct TestFixture {
    pub worker: Worker<Sandbox>,
    pub contract: Contract,
    pub near_client: ContractClient,
}

impl TestFixture {
    pub async fn new() -> Result<Self> {
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

        // Create our NearContract wrapper with default config for tests
        let relayer_config = RelayerConfig::default();
        let near_client = ContractClient::new(contract.id().clone(), signer, near_fetch_client, relayer_config);

        Ok(Self {
            worker,
            contract,
            near_client,
        })
    }

    /// Initialize the contract with Sepolia test data
    pub async fn init_with_sepolia(&self) -> Result<InitInput> {
        let init_input = load_sepolia_init_data()?;
        self.near_client.init_contract(init_input.clone()).await?;
        Ok(init_input)
    }

    /// Initialize the contract with Sepolia test data but skip validation
    pub async fn init_with_sepolia_no_validation(&self) -> Result<InitInput> {
        let mut init_input = load_sepolia_init_data()?;
        init_input.validate_updates = false;
        init_input.verify_bls_signatures = false;
        self.near_client.init_contract(init_input.clone()).await?;
        Ok(init_input)
    }
}

/// Simple helper to load Sepolia test data
pub fn load_sepolia_init_data() -> Result<InitInput> {
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
        finalized_beacon_header: first_update.finalized_header.clone().into(),
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

/// Load test execution headers
pub fn load_test_headers() -> Result<Vec<BlockHeader>> {
    let headers: Vec<BlockHeader> = serde_json::from_reader(std::fs::File::open(
        "./tests/data/execution_blocks_8286935_8295112.json",
    )?)?;
    Ok(headers)
}

/// Load test light client updates
pub fn load_test_light_client_updates() -> Result<(LightClientUpdate, LightClientUpdate)> {
    let init_update: LightClientUpdate = serde_json::from_reader(std::fs::File::open(
        "./tests/data/light_client_update_period_925.json",
    )?)?;

    let first_update: LightClientUpdate = serde_json::from_reader(std::fs::File::open(
        "./tests/data/light_client_update_period_926.json",
    )?)?;

    Ok((init_update, first_update))
}
