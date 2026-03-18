#![allow(dead_code)] // Allow unused functions in test utilities

use color_eyre::Result;
use color_eyre::eyre::Context;
use eth_types::BlockHeader;
use eth_types::eth2::LightClientUpdate;
use eth2_utility::consensus::Network;
use eth2_utility::types::InitInput;
use near_crypto::{InMemorySigner, SecretKey};
use near_workspaces::network::Sandbox;
use near_workspaces::{Contract, Worker, cargo_near_build};
use relayer::config::NearConfig;
use relayer::{ContractClient, config::RelayerConfig};

/// Test fixture that sets up the sandbox environment and deploys the contract
pub struct TestFixture {
    pub worker: Worker<Sandbox>,
    pub contract: Contract,
    pub near_client: ContractClient,
    pub near_client_with_contract_signer: ContractClient,
    relayer_account_id: near_workspaces::AccountId,
}

impl TestFixture {
    pub async fn new() -> Result<Self> {
        // Install color-eyre, ignoring error if already installed
        let _ = color_eyre::install();

        // Compile the eth2-client
        let wasm = compile_eth2_client_testnet()
            .await
            .wrap_err("Failed to compile eth2-client contract")?;

        // Create sandbox environment
        let worker = near_workspaces::sandbox()
            .await
            .wrap_err("Failed to create sandbox environment")?;

        let (contract_id, contract_secret_key) = worker.generate_dev_account_credentials();

        // Deploy the contract
        let contract = worker
            .create_root_account_subaccount_and_deploy(
                contract_id.clone(),
                contract_secret_key.clone(),
                &wasm,
            )
            .await?
            .into_result()?;

        let contract_signer = InMemorySigner::from_secret_key(
            contract.id().clone(),
            contract_secret_key
                .to_string()
                .parse()
                .wrap_err("Failed to parse secret key")?,
        );

        let alice = worker
            .dev_create_account()
            .await
            .wrap_err("Failed to create test account")?;

        let alice_secret_key: SecretKey = alice
            .secret_key()
            .to_string()
            .parse()
            .wrap_err("Failed to parse secret key")?;

        let alice_signer =
            InMemorySigner::from_secret_key(alice.id().clone(), alice_secret_key.clone());

        // Create the near-fetch client pointing to the sandbox RPC
        let near_fetch_client = near_fetch::Client::new(&worker.rpc_addr());

        // Create our NearContract wrapper with default config for tests
        let relayer_config = RelayerConfig::default();
        let near_config = NearConfig::default();
        let near_client = ContractClient::new(
            contract.id().clone(),
            alice_signer,
            near_fetch_client.clone(),
            relayer_config.clone(),
            near_config.timeout_secs,
        );

        let near_client_with_contract_signer = ContractClient::new(
            contract.id().clone(),
            contract_signer,
            near_fetch_client,
            relayer_config,
            near_config.timeout_secs,
        );

        Ok(Self {
            worker,
            contract,
            near_client,
            near_client_with_contract_signer,
            relayer_account_id: alice.id().clone(),
        })
    }

    /// Initialize the contract with Sepolia test data
    pub async fn init_with_sepolia(&self) -> Result<InitInput> {
        let init_input = load_sepolia_init_data()?;
        self.near_client_with_contract_signer
            .init_contract(init_input.clone())
            .await?;
        self.grant_relayer_role().await?;
        Ok(init_input)
    }

    /// Initialize the contract with Sepolia test data but skip validation
    pub async fn init_with_sepolia_no_validation(&self) -> Result<InitInput> {
        let mut init_input = load_sepolia_init_data()?;
        init_input.validate_updates = false;
        init_input.verify_bls_signatures = false;
        self.near_client_with_contract_signer
            .init_contract(init_input.clone())
            .await?;
        self.grant_relayer_role().await?;
        Ok(init_input)
    }

    /// Grant the relayer's signer the bypass roles so it passes the
    /// trusted_relayer guard on submit methods.
    /// Must be called after init, since init resets ACL state via acl_init_super_admin.
    async fn grant_relayer_role(&self) -> Result<()> {
        self.contract
            .call("acl_grant_role")
            .args_json(serde_json::json!({
                "role": "UnrestrictedSubmitLightClientUpdate",
                "account_id": self.relayer_account_id.to_string(),
            }))
            .transact()
            .await?
            .into_result()
            .wrap_err("Failed to grant UnrestrictedSubmitLightClientUpdate role")?;
        self.contract
            .call("acl_grant_role")
            .args_json(serde_json::json!({
                "role": "UnrestrictedSubmitExecutionHeader",
                "account_id": self.relayer_account_id.to_string(),
            }))
            .transact()
            .await?
            .into_result()
            .wrap_err("Failed to grant UnrestrictedSubmitExecutionHeader role")?;
        Ok(())
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

// near_workspaces::compile_project
pub async fn compile_eth2_client_testnet() -> crate::Result<Vec<u8>> {
    let project_path = "../contracts/near/eth2-client";
    let project_path =
        std::fs::canonicalize(project_path).wrap_err("Failed to parse eth2-client path")?;

    // `no_abi` has become flipped true -> false
    let cargo_opts = cargo_near_build::BuildOpts {
        no_locked: true,
        manifest_path: Some(
            cargo_near_build::camino::Utf8PathBuf::from_path_buf(project_path.join("Cargo.toml"))
                .map_err(|e| color_eyre::Report::msg(format!("{e:?}")))?,
        ),
        no_default_features: true,
        features: Some("logs".to_string()),
        ..Default::default()
    };

    let compile_artifact =
        cargo_near_build::build_with_cli(cargo_opts).wrap_err("Fail on build with cli")?;

    let file = compile_artifact
        .canonicalize()
        .wrap_err("Fail to canonicalize build artifact")?;
    tokio::fs::read(file)
        .await
        .wrap_err("Fail to read build artifact")
}
