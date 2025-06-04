#[cfg(test)]
mod integration_tests {
    use crate::tests::utils::*;
    use borsh::{BorshDeserialize, BorshSerialize};
    use eth2_utility::types::InitInput;
    use eth_types::eth2::{FinalizedHeader, SyncCommittee};
    use eth_types::{Address, Bloom, H256, H64, U256};
    use near_sdk::{Gas, NearToken};
    use near_workspaces::operations::Function;
    use near_workspaces::{Account, Contract};
    use serde::{Deserialize, Serialize};

    const WASM_FILEPATH: &str = "../target/near/eth2_client/eth2_client.wasm";
    const WASM_V_0_1_0_FILEPATH: &str = "src/data/eth2_client_v0.1.0_testnet.wasm";
    #[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
    pub struct BlockHeaderV1 {
        pub parent_hash: H256,
        pub uncles_hash: H256,
        pub author: Address,
        pub state_root: H256,
        pub transactions_root: H256,
        pub receipts_root: H256,
        pub log_bloom: Bloom,
        pub difficulty: U256,
        #[serde(with = "serde_utils::u64_hex_be")]
        pub number: u64,
        pub gas_limit: U256,
        pub gas_used: U256,
        #[serde(with = "serde_utils::u64_hex_be")]
        pub timestamp: u64,
        #[serde(with = "serde_utils::hex_vec")]
        pub extra_data: Vec<u8>,
        pub mix_hash: H256,
        pub nonce: H64,
        #[serde(with = "eth_types::u64_hex_be_option")]
        pub base_fee_per_gas: Option<u64>,
        pub withdrawals_root: Option<H256>,

        pub hash: Option<H256>,
        pub partial_hash: Option<H256>,
    }

    impl From<eth_types::BlockHeader> for BlockHeaderV1 {
        fn from(item: eth_types::BlockHeader) -> Self {
            serde_json::from_str(&serde_json::to_string(&item).unwrap()).unwrap()
        }
    }

    #[derive(Clone, BorshDeserialize, BorshSerialize)]
    pub struct InitInputV1 {
        pub network: String,
        pub finalized_execution_header: BlockHeaderV1,
        pub finalized_beacon_header: FinalizedHeader,
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
                finalized_execution_header: message.finalized_execution_header.into(),
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
        let worker = near_workspaces::sandbox().await?;
        let wasm = std::fs::read(file_path)?;
        let contract = worker.dev_deploy(&wasm).await?;

        // create accounts
        let owner = worker.root_account()?;
        let alice = owner
            .create_subaccount("alice")
            .initial_balance(NearToken::from_near(30))
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
        update.finalized_header.execution.block_hash =
            headers.last().unwrap().calculate_hash();
        let outcome = alice
            .call(contract.id(), "submit_beacon_chain_light_client_update")
            .args_borsh(update)
            .gas(Gas::from_tgas(300))
            .transact()
            .await?;
        assert!(outcome.is_success());

        for headers_chunk in headers.iter().rev().collect::<Vec<_>>().chunks(50) {
            let mut transaction = alice.batch(contract.id());
            for header in headers_chunk {
                transaction = transaction.call(
                    Function::new("submit_execution_header")
                        .args(borsh::to_vec(&header)?)
                        .gas(Gas::from_tgas(6)),
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
        println!("Gas burnt: {}", outcome.total_gas_burnt);
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
            .deposit(NearToken::from_near(20))
            .transact()
            .await?;
        assert!(result.is_success());

        // Submit blocks [1..num_of_blocks_to_submit]
        for headers_chunk in headers.chunks(50) {
            let mut transaction = alice.batch(contract.id());
            for header in headers_chunk {
                let header_v1: BlockHeaderV1 = header.clone().into();
                transaction = transaction.call(
                    Function::new("submit_execution_header")
                        .args(borsh::to_vec(&header_v1)?)
                        .gas(Gas::from_tgas(6)),
                );
            }

            let result = transaction.transact().await?;
            assert!(result.is_success());
        }

        // Submit light client update and finilized submited blocks
        let mut update = updates[1].clone();
        update.finalized_header.execution.block_hash =
            headers.last().unwrap().calculate_hash();
        let outcome = alice
            .call(contract.id(), "submit_beacon_chain_light_client_update")
            .args_borsh(update)
            .gas(Gas::from_tgas(300))
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
            .gas(Gas::from_tgas(300))
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
        update.finalized_header.execution.block_hash =
            headers.last().unwrap().calculate_hash();

        // Submit light client update
        let result = alice
            .call(contract.id(), "submit_beacon_chain_light_client_update")
            .args_borsh(update)
            .gas(Gas::from_tgas(300))
            .transact()
            .await?;
        assert!(result.is_success());

        // Submit and finilize blocks [num_of_blocks_to_submit..num_of_blocks_to_submit*2]
        for headers_chunk in headers.iter().rev().collect::<Vec<_>>().chunks(50) {
            let mut transaction = alice.batch(contract.id());
            for header in headers_chunk {
                transaction = transaction.call(
                    Function::new("submit_execution_header")
                        .args(borsh::to_vec(header)?)
                        .gas(Gas::from_tgas(6)),
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
