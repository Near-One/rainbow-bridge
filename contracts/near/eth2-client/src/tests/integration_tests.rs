#[cfg(test)]
mod integration_tests {
    use crate::tests::utils::*;
    use borsh::BorshSerialize;
    use eth2_utility::types::InitInput;
    use eth_types::H256;
    use near_sdk::ONE_NEAR;
    use near_units::*;
    use workspaces::operations::Function;
    use workspaces::prelude::*;
    use workspaces::{network::Sandbox, Account, Contract, Worker};

    const WASM_FILEPATH: &str = "../res/eth2_client.wasm";

    async fn initialize_client(
        init_input: InitInput,
    ) -> anyhow::Result<(Account, Contract, Worker<Sandbox>)> {
        let worker = workspaces::sandbox().await?;
        let wasm = std::fs::read(WASM_FILEPATH)?;
        let contract = worker.dev_deploy(&wasm).await?;

        // create accounts
        let owner = worker.root_account()?;
        let alice = owner
            .create_subaccount(&worker, "alice")
            .initial_balance(parse_near!("30 N"))
            .transact()
            .await?
            .into_result()?;

        alice
            .call(&worker, contract.id(), "init")
            .args(init_input.try_to_vec()?)
            .transact()
            .await?;
        Ok((alice, contract, worker))
    }

    #[tokio::test]
    async fn test_gas_usage_of_submit_beacon_chain_light_client_update() -> anyhow::Result<()> {
        let (headers, updates, init_input) = get_kiln_test_data(Some(InitOptions {
            validate_updates: false,
            verify_bls_signatures: false,
            hashes_gc_threshold: 51000,
            max_submitted_blocks_by_account: 7000,
            trusted_signer: None,
        }));
        let (alice, contract, worker) = initialize_client(init_input).await?;

        alice
            .call(&worker, contract.id(), "register_submitter")
            .deposit(10 * ONE_NEAR)
            .transact()
            .await?;

        let num_of_blocks_to_submit = 32;
        let headers = &headers.as_slice()[1..num_of_blocks_to_submit];
        for headers_chunk in headers.chunks(50) {
            let mut transaction = alice.batch(&worker, contract.id());
            for header in headers_chunk {
                transaction = transaction.call(
                    Function::new("submit_execution_header")
                        .args(header.try_to_vec()?)
                        .gas(parse_gas!("6 T") as u64),
                );
            }

            transaction.transact().await?;
        }

        let mut update = updates[1].clone();
        update.finality_update.header_update.execution_block_hash =
            headers.last().unwrap().calculate_hash();
        let outcome = alice
            .call(
                &worker,
                contract.id(),
                "submit_beacon_chain_light_client_update",
            )
            .args(update.try_to_vec()?)
            .gas(parse_gas!("300 T") as u64)
            .transact()
            .await?;

        for header in headers {
            let result: Option<H256> = contract
                .view(&worker, "block_hash_safe", header.number.try_to_vec()?)
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
}
