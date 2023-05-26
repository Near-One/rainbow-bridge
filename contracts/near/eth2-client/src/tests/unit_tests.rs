#[cfg(test)]
mod tests {
    use crate::tests::utils::*;
    use crate::{Eth2Client, PAUSE_SUBMIT_UPDATE};
    use admin_controlled::AdminControlled;
    use bitvec::bitarr;
    use bitvec::order::Lsb0;
    use eth2_utility::consensus::*;
    use eth_types::eth2::LightClientUpdate;
    use eth_types::{BlockHeader, H256, U256};
    use hex::FromHex;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, VMConfig};
    use tree_hash::TreeHash;

    macro_rules! inner_set_env {
        ($builder:ident) => {
            $builder
        };

        ($builder:ident, $key:ident:$value:expr $(,$key_tail:ident:$value_tail:expr)*) => {
            {
            $builder.$key($value.try_into().unwrap());
            inner_set_env!($builder $(,$key_tail:$value_tail)*)
            }
        };
    }

    macro_rules! set_env {
        ($($key:ident:$value:expr),* $(,)?) => {
            let mut builder = VMContextBuilder::new();
            let mut builder = &mut builder;
            builder = inner_set_env!(builder, $($key: $value),*);
            let mut vm_config = VMConfig::free();
            vm_config.limit_config.max_number_logs = u64::MAX;
            vm_config.limit_config.max_total_log_length = u64::MAX;
            testing_env!(builder.build(), vm_config);
        };
    }

    pub struct TestContext<'a> {
        contract: Eth2Client,
        headers: &'a Vec<Vec<BlockHeader>>,
        updates: &'a Vec<LightClientUpdate>,
    }

    pub fn get_test_context(init_options: Option<InitOptions>) -> TestContext<'static> {
        let (headers, updates, init_input) = get_test_data(init_options);
        let contract = Eth2Client::init(init_input);
        assert_eq!(contract.last_block_number(), headers[0][0].number);

        TestContext {
            contract,
            headers: &headers,
            updates: &updates,
        }
    }

    pub fn submit_and_check_execution_headers(
        contract: &mut Eth2Client,
        headers: Vec<&BlockHeader>,
    ) {
        for header in headers {
            contract.submit_execution_header(header.clone());
            assert!(contract.is_known_execution_header(header.number));
        }
    }

    #[cfg(not(feature = "mainnet"))]
    mod generic_tests {
        use super::*;

        #[test]
        pub fn test_header_root() {
            let header =
                read_beacon_header(format!("./src/data/goerli/beacon_header_{}.json", 5258752));
            assert_eq!(
                H256(header.tree_hash_root()),
                Vec::from_hex("cd669c0007ab6ff261a02cc3335ba470088e92f0460bf1efac451009efb9ec0a")
                    .unwrap()
                    .into()
            );

            let header =
                read_beacon_header(format!("./src/data/mainnet/beacon_header_{}.json", 4100000));
            assert_eq!(
                H256(header.tree_hash_root()),
                Vec::from_hex("342ca1455e976f300cc96a209106bed2cbdf87243167fab61edc6e2250a0be6c")
                    .unwrap()
                    .into()
            );
        }

        #[test]
        pub fn test_submit_update_two_periods() {
            let submitter = accounts(0);
            let TestContext {
                mut contract,
                headers,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter);

            contract.submit_beacon_chain_light_client_update(updates[1].clone());

            submit_and_check_execution_headers(
                &mut contract,
                headers[0].iter().skip(1).rev().collect(),
            );

            for header in headers[0].iter().skip(1) {
                let header_hash = header.calculate_hash();
                assert!(
                    contract.block_hash_safe(header.number).unwrap_or_default() == header_hash,
                    "Execution block hash is not finalized: {:?}",
                    header_hash
                );
            }

            assert_eq!(
                contract.last_block_number(),
                headers[0].last().unwrap().number
            );
        }

        #[test]
        #[should_panic(expected = "The expected block hash is")]
        pub fn test_panic_on_submit_execution_block_from_fork_chain() {
            let submitter = accounts(0);
            let TestContext {
                mut contract,
                headers,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter);
            contract.submit_beacon_chain_light_client_update(updates[1].clone());

            // Submit execution header with different hash
            let mut fork_header = headers[0][1].clone();
            // Difficulty is modified just in order to get a different header hash. Any other field would be suitable too
            fork_header.difficulty = U256::from(ethereum_types::U256::from(99));
            contract.submit_execution_header(fork_header.clone());
        }

        #[test]
        pub fn test_gc_headers() {
            let submitter = accounts(0);
            let hashes_gc_threshold: usize = 9500;
            let TestContext {
                mut contract,
                headers,
                updates,
            } = get_test_context(Some(InitOptions {
                validate_updates: true,
                verify_bls_signatures: true,
                hashes_gc_threshold: hashes_gc_threshold.try_into().unwrap(),
                trusted_signer: None,
            }));
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter);

            contract.submit_beacon_chain_light_client_update(updates[1].clone());

            submit_and_check_execution_headers(
                &mut contract,
                headers[0].iter().skip(1).rev().collect(),
            );

            // Execution headers are finalized
            for header in headers[0].iter().skip(1) {
                assert!(
                    contract.block_hash_safe(header.number).unwrap_or_default()
                        == header.calculate_hash(),
                    "Execution block is not finalized: {:?}",
                    header.number
                );
            }

            contract.submit_beacon_chain_light_client_update(updates[2].clone());
            submit_and_check_execution_headers(&mut contract, headers[1].iter().rev().collect());

            assert_eq!(
                contract.last_block_number(),
                headers[1].last().unwrap().number
            );

            // Execution headers are finalized
            for header in headers[1].iter() {
                assert!(
                    contract.block_hash_safe(header.number).unwrap_or_default()
                        == header.calculate_hash(),
                    "Execution block is not finalized: {:?}",
                    header.number
                );
            }

            // Headers older than the hashes_gc_threshold headers are both removed and are not present in execution header list
            for header in headers.concat().iter().rev().skip(hashes_gc_threshold + 2) {
                assert!(
                    contract.block_hash_safe(header.number).is_none(),
                    "Execution block was not removed: {:?}",
                    header.number
                );
            }
        }

        #[test]
        #[should_panic(expected = "only trusted_signer can update the client")]
        pub fn test_trusted_signer() {
            let trusted_signer = accounts(1);
            let TestContext {
                mut contract,
                headers: _,
                updates,
            } = get_test_context(Some(InitOptions {
                validate_updates: true,
                verify_bls_signatures: true,
                hashes_gc_threshold: 7100,
                trusted_signer: Some(trusted_signer),
            }));
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
            contract.submit_beacon_chain_light_client_update(updates[1].clone());
        }

        #[test]
        #[should_panic(expected = "Invalid finality proof")]
        pub fn test_panic_on_invalid_finality_proof() {
            let TestContext {
                mut contract,
                headers: _,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
            let mut update = updates[1].clone();
            update.finality_update.finality_branch[5] = H256::from(
                hex::decode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                    .unwrap(),
            );
            contract.submit_beacon_chain_light_client_update(update);
        }

        #[test]
        #[should_panic(expected = "Invalid finality proof")]
        pub fn test_panic_on_empty_finality_proof() {
            let TestContext {
                mut contract,
                headers: _,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
            let mut update = updates[1].clone();
            update.finality_update.finality_branch = vec![];
            contract.submit_beacon_chain_light_client_update(update);
        }

        #[test]
        #[should_panic(expected = "Invalid execution block hash proof")]
        pub fn test_panic_on_invalid_execution_block_proof() {
            let TestContext {
                mut contract,
                headers: _,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
            let mut update = updates[1].clone();
            update.finality_update.header_update.execution_hash_branch[5] = H256::from(
                hex::decode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                    .unwrap(),
            );
            contract.submit_beacon_chain_light_client_update(update);
        }

        #[test]
        #[should_panic(expected = "Invalid execution block hash proof")]
        pub fn test_panic_on_empty_execution_block_proof() {
            let TestContext {
                mut contract,
                headers: _,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
            let mut update = updates[1].clone();
            update.finality_update.header_update.execution_hash_branch = vec![];
            contract.submit_beacon_chain_light_client_update(update);
        }

        #[test]
        #[should_panic(expected = "The acceptable update periods are")]
        pub fn test_panic_on_skip_update_period() {
            let TestContext {
                mut contract,
                headers: _,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
            let mut update = updates[1].clone();
            update.finality_update.header_update.beacon_header.slot =
                update.signature_slot + EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH * 10;
            update.attested_beacon_header.slot =
                update.finality_update.header_update.beacon_header.slot;
            update.signature_slot = update.attested_beacon_header.slot + 1;
            contract.submit_beacon_chain_light_client_update(update);
        }

        #[test]
        #[should_panic(expected = "The expected block hash is")]
        pub fn test_panic_on_submit_update_with_missing_execution_blocks() {
            let submitter = accounts(0);
            let TestContext {
                mut contract,
                headers,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter);

            contract.submit_beacon_chain_light_client_update(updates[1].clone());

            submit_and_check_execution_headers(
                &mut contract,
                headers[0].iter().skip(1).take(5).collect(),
            );
        }

        #[test]
        #[should_panic(expected = "The expected block hash is")]
        pub fn test_panic_on_submit_same_execution_blocks() {
            let submitter = accounts(0);
            let TestContext {
                mut contract,
                headers,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter);

            contract.submit_beacon_chain_light_client_update(updates[1].clone());
            contract.submit_execution_header(headers[0].last().unwrap().clone());
            contract.submit_execution_header(headers[0].last().unwrap().clone());
        }

        #[test]
        #[should_panic(expected = "paused")]
        pub fn test_panic_on_submit_update_paused() {
            let TestContext {
                mut contract,
                headers: _,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0), current_account_id: accounts(0));
            contract.set_paused(PAUSE_SUBMIT_UPDATE);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(1), current_account_id: accounts(0));
            contract.submit_beacon_chain_light_client_update(updates[1].clone());
        }

        #[test]
        #[should_panic(
            expected = "The active header slot number should be higher than the finalized slot"
        )]
        pub fn test_panic_on_submit_outdated_update() {
            let submitter = accounts(0);
            let TestContext {
                mut contract,
                headers: _,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter);

            contract.submit_beacon_chain_light_client_update(updates[0].clone());
        }

        #[test]
        #[should_panic(expected = "The expected block hash")]
        pub fn test_panic_on_submit_blocks_with_unknown_parent() {
            let submitter = accounts(0);
            let TestContext {
                mut contract,
                headers,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter);

            assert_eq!(contract.last_block_number(), headers[0][0].number);

            contract.submit_beacon_chain_light_client_update(updates[1].clone());

            let headers: Vec<_> = headers.iter().skip(1).rev().collect();
            contract.submit_execution_header(headers[0][0].clone());
            // Skip 2th block
            contract.submit_execution_header(headers[0][3].clone());
        }

        #[test]
        #[should_panic(expected = "== ClientMode::SubmitHeader")]
        pub fn test_panic_on_submit_headers_in_worng_mode() {
            let submitter = accounts(0);
            let TestContext {
                mut contract,
                headers,
                updates: _,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: submitter);

            assert_eq!(contract.last_block_number(), headers[0][0].number);

            contract.submit_execution_header(headers[0][1].clone());
        }

        #[test]
        #[should_panic(
            expected = "Sync committee bits sum is less than 2/3 threshold, bits sum: 341"
        )]
        pub fn test_panic_on_sync_committee_bits_is_less_than_threshold() {
            let TestContext {
                mut contract,
                headers: _,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
            let mut update = updates[1].clone();

            let mut sync_committee_bits = bitarr![u8, Lsb0; 0; 512];

            // The number of participants should satisfy the inequality:
            // num_of_participants * 3 >= sync_committee_bits_size * 2
            // If the sync_committee_bits_size = 512, then
            // the minimum allowed value of num_of_participants is 342.

            // Fill the sync_committee_bits with 341 participants to trigger panic
            let num_of_participants = (((512.0 * 2.0 / 3.0) as f32).ceil() - 1.0) as usize;
            sync_committee_bits
                .get_mut(0..num_of_participants)
                .unwrap()
                .fill(true);
            update.sync_aggregate.sync_committee_bits =
                sync_committee_bits.as_raw_mut_slice().to_vec().into();
            contract.submit_beacon_chain_light_client_update(update);
        }

        #[test]
        #[should_panic(expected = "The sync committee update is missed")]
        pub fn test_panic_on_missing_sync_committee_update() {
            let TestContext {
                mut contract,
                headers: _,
                updates,
            } = get_test_context(None);
            set_env!(prepaid_gas: 10u64.pow(18), predecessor_account_id: accounts(0));
            let mut update = updates[1].clone();
            update.sync_committee_update = None;
            contract.submit_beacon_chain_light_client_update(update);
        }
    }

    #[cfg(feature = "mainnet")]
    mod mainnet_tests {
        use super::*;

        #[test]
        #[should_panic(
            expected = "The client can't be executed in the trustless mode without BLS sigs verification on Mainnet"
        )]
        pub fn test_panic_on_init_in_trustless_mode_without_bls_on_mainnet() {
            let (_headers, _updates, init_input) = get_test_data(Some(InitOptions {
                validate_updates: true,
                verify_bls_signatures: false,
                hashes_gc_threshold: 500,
                trusted_signer: None,
            }));

            Eth2Client::init(init_input);
        }

        #[test]
        #[cfg_attr(feature = "bls", ignore)]
        #[should_panic(
            expected = "The client can't be executed in the trustless mode without BLS sigs verification on Mainnet"
        )]
        pub fn test_panic_on_init_in_trustless_mode_without_bls_feature_flag() {
            let (_headers, _updates, init_input) = get_test_data(Some(InitOptions {
                validate_updates: true,
                verify_bls_signatures: true,
                hashes_gc_threshold: 500,
                trusted_signer: None,
            }));

            Eth2Client::init(init_input);
        }
    }
}
