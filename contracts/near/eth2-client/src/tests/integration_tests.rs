#[cfg(test)]
mod integration_tests {
    use crate::tests::utils::*;
    use borsh::{BorshDeserialize, BorshSerialize};
    use eth2_utility::types::InitInput;
    use eth_types::eth2::{ExtendedBeaconBlockHeader, SyncCommittee};
    use eth_types::H256;
    use near_sdk::ONE_NEAR;
    use near_units::*;
    use workspaces::operations::Function;
    use workspaces::{Account, Contract};

    const WASM_FILEPATH: &str = "../target/wasm32-unknown-unknown/release/eth2_client.wasm";
    const WASM_V_0_1_0_FILEPATH: &str = "src/data/eth2_client_v0.1.0_testnet.wasm";

    #[derive(Clone, BorshDeserialize, BorshSerialize)]
    pub struct InitInputV1 {
        pub network: String,
        pub finalized_execution_header: eth_types::BlockHeader,
        pub finalized_beacon_header: ExtendedBeaconBlockHeader,
        pub current_sync_committee: SyncCommittee,
        pub next_sync_committee: SyncCommittee,
        pub validate_updates: bool,
        pub verify_bls_signatures: bool,
        pub hashes_gc_threshold: u64,
        pub max_submitted_blocks_by_account: u32,
        pub trusted_signer: Option<near_sdk::AccountId>,
    }

    impl From<InitInput> for InitInputV1 {
        fn from(message: InitInput) -> Self {
            Self {
                network: message.network,
                finalized_execution_header: message.finalized_execution_header,
                finalized_beacon_header: message.finalized_beacon_header,
                current_sync_committee: message.current_sync_committee,
                next_sync_committee: message.next_sync_committee,
                validate_updates: message.validate_updates,
                verify_bls_signatures: message.verify_bls_signatures,
                hashes_gc_threshold: message.hashes_gc_threshold,
                max_submitted_blocks_by_account: 10000,
                trusted_signer: message.trusted_signer,
            }
        }
    }

    async fn initialize_client<U: borsh::BorshSerialize>(
        init_input: U,
        file_path: &str,
    ) -> anyhow::Result<(Account, Contract)> {
        let worker = workspaces::sandbox().await?;
        let wasm = std::fs::read(file_path)?;
        let contract = worker.dev_deploy(&wasm).await?;

        // create accounts
        let owner = worker.root_account()?;
        let alice = owner
            .create_subaccount("alice")
            .initial_balance(parse_near!("30 N"))
            .transact()
            .await?
            .into_result()?;

        let result = contract
            .call("init")
            .args_borsh(init_input)
            .transact()
            .await?;

        assert!(result.is_success());

        Ok((alice, contract))
    }

    #[tokio::test]
    async fn test_gas_usage_of_submit_beacon_chain_light_client_update() -> anyhow::Result<()> {
        let (headers, updates, init_input) = get_goerli_test_data(Some(InitOptions {
            validate_updates: false,
            verify_bls_signatures: false,
            hashes_gc_threshold: 51000,
            trusted_signer: None,
        }));
        let (alice, contract) = initialize_client(init_input, WASM_FILEPATH).await?;
        let num_of_blocks_to_submit = 32;
        let headers = headers[0].as_slice()[1..num_of_blocks_to_submit].to_vec();

        let mut update = updates[1].clone();
        update.finality_update.header_update.execution_block_hash =
            headers.last().unwrap().calculate_hash();
        let outcome = alice
            .call(contract.id(), "submit_beacon_chain_light_client_update")
            .args_borsh(update)
            .gas(parse_gas!("300 T") as u64)
            .transact()
            .await?;
        assert!(outcome.is_success());

        for headers_chunk in headers.iter().rev().collect::<Vec<_>>().chunks(50) {
            let mut transaction = alice.batch(contract.id());
            for header in headers_chunk {
                transaction = transaction.call(
                    Function::new("submit_execution_header")
                        .args(header.try_to_vec()?)
                        .gas(parse_gas!("6 T") as u64),
                );
            }

            let result = transaction.transact().await?;
            assert!(result.is_success());
        }

        for header in headers {
            let result: Option<H256> = contract
                .view("block_hash_safe")
                .args_borsh(header.number)
                .await?
                .borsh()?;
            assert!(result.is_some())
        }
        println!(
            "Gas burnt: {}",
            gas::to_human(outcome.total_gas_burnt as u128)
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_migration() -> anyhow::Result<()> {
        let (headers_data, updates, init_input) = get_goerli_test_data(Some(InitOptions {
            validate_updates: false,
            verify_bls_signatures: false,
            hashes_gc_threshold: 51000,
            trusted_signer: None,
        }));
        let init_input: InitInputV1 = init_input.into();
        let (alice, contract) = initialize_client(init_input, WASM_V_0_1_0_FILEPATH).await?;
        let num_of_blocks_to_submit = 32;
        let headers = headers_data[0].as_slice()[1..num_of_blocks_to_submit].to_vec();

        let result = alice
            .call(contract.id(), "register_submitter")
            .deposit(20 * ONE_NEAR)
            .transact()
            .await?;
        assert!(result.is_success());

        // Submit blocks [1..num_of_blocks_to_submit]
        for headers_chunk in headers.chunks(50) {
            let mut transaction = alice.batch(contract.id());
            for header in headers_chunk {
                transaction = transaction.call(
                    Function::new("submit_execution_header")
                        .args(header.try_to_vec()?)
                        .gas(parse_gas!("6 T") as u64),
                );
            }

            let result = transaction.transact().await?;
            assert!(result.is_success());
        }

        // Submit light client update and finilized submited blocks
        let mut update = updates[1].clone();
        update.finality_update.header_update.execution_block_hash =
            headers.last().unwrap().calculate_hash();
        let outcome = alice
            .call(contract.id(), "submit_beacon_chain_light_client_update")
            .args_borsh(update)
            .gas(parse_gas!("300 T") as u64)
            .transact()
            .await?;
        assert!(outcome.is_success());

        // Verify finilized blocks
        for header in &headers {
            let result: Option<H256> = contract
                .view("block_hash_safe")
                .args_borsh(header.number)
                .await?
                .borsh()?;
            assert!(result.is_some())
        }

        // Deploy new version
        let contract = contract
            .as_account()
            .deploy(&(std::fs::read(WASM_FILEPATH).unwrap()))
            .await
            .unwrap()
            .result;

        // Migrate
        let result = contract
            .call("migrate")
            .gas(parse_gas!("300 T") as u64)
            .transact()
            .await?;
        assert!(result.is_success());

        // Verify finilized blocks after migration
        for header in headers {
            let result: Option<H256> = contract
                .view("block_hash_safe")
                .args_borsh(header.number)
                .await?
                .borsh()?;
            assert!(result.is_some())
        }

        let headers = headers_data[0].as_slice()
            [num_of_blocks_to_submit..num_of_blocks_to_submit * 2]
            .to_vec();
        let mut update = updates[2].clone();
        update.finality_update.header_update.execution_block_hash =
            headers.last().unwrap().calculate_hash();

        // Submit light client update
        let result = alice
            .call(contract.id(), "submit_beacon_chain_light_client_update")
            .args_borsh(update)
            .gas(parse_gas!("300 T") as u64)
            .transact()
            .await?;
        assert!(result.is_success());

        // Submit and finilize blocks [num_of_blocks_to_submit..num_of_blocks_to_submit*2]
        for headers_chunk in headers.iter().rev().collect::<Vec<_>>().chunks(50) {
            let mut transaction = alice.batch(contract.id());
            for header in headers_chunk {
                transaction = transaction.call(
                    Function::new("submit_execution_header")
                        .args(header.try_to_vec()?)
                        .gas(parse_gas!("6 T") as u64),
                );
            }

            let result = transaction.transact().await?;
            assert!(result.is_success());
        }

        // Verify finilized blocks [1..num_of_blocks_to_submit*2]
        let headers = headers_data[0].as_slice()[1..num_of_blocks_to_submit * 2].to_vec();
        for header in headers {
            let result: Option<H256> = contract
                .view("block_hash_safe")
                .args_borsh(header.number)
                .await?
                .borsh()?;
            assert!(result.is_some())
        }

        Ok(())
    }
}
