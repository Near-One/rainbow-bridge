#![feature(const_vec_new)]

extern crate crypto;
extern crate rlp;

use std::collections::HashMap;
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::{near_bindgen};

mod header;
use header::{BlockHeader};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct EthBridge {
    block_hashes: HashMap<u64, [u8; 32]>,
    last_block_number: u64,
}

#[near_bindgen]
impl EthBridge {

    const NUMBER_OF_FUTURE_BLOCKS: u64 = 10;

    pub fn add_block_headers(&mut self, start: u64, block_headers: Vec<Vec<u8>>) {

        for i in 0..block_headers.len() as u64 {
            
            match rlp::decode::<BlockHeader>(block_headers[i as usize].as_slice()) {
                Ok(header) => {

                    // Check prev block compatibility
                    match self.block_hash(i - 1) {
                        Some(hash) => {
                            if self.last_block_number > 0 {
                                assert_eq!(header.number - 1, start - 1 + i);
                            }
                            assert_eq!(header.parent_hash, hash.into());
                        },
                        None => if i > 0 { panic!() },
                    }

                    // Do not add latest 10 blocks, only check em
                    if i + EthBridge::NUMBER_OF_FUTURE_BLOCKS < block_headers.len() as u64 {

                        self.block_hashes.insert(start + i, header.hash.into());

                        // Update self.last_block_number only once
                        if i + EthBridge::NUMBER_OF_FUTURE_BLOCKS == block_headers.len() as u64 - 1 {

                            // Check longest chain rule
                            if header.number < self.last_block_number {
                                panic!(); // revert all state changes
                            }
                            self.last_block_number = header.number;
                        }
                    }
                },
                Err(_e) => {
                    panic!();
                },
            }
        }
    }

    pub fn block_hash(&self, index: u64) -> Option<[u8; 32]> {
        if index + EthBridge::NUMBER_OF_FUTURE_BLOCKS > self.last_block_number {
            return Option::None;
        }
        self.block_hashes.get(&index).cloned()
    }
}

#[cfg(feature = "env_test")]
#[cfg(test)]
mod tests {
    use super::*;
    use near_bindgen::MockedBlockchain;
    use near_bindgen::{VMContext, Config, testing_env};

    fn get_context(input: Vec<u8>) -> VMContext {
        VMContext {
            current_account_id: "alice.near".to_string(),
            signer_account_id: "bob.near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol.near".to_string(),
            input,
            block_index: 0,
            account_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(9),
            random_seed: vec![0, 1, 2],
            free_of_charge: false,
            output_data_receivers: vec![],
        }
    }

    #[test]
    fn set_get_message() {
        let context = get_context(vec![]);
        let config = Config::default();
        testing_env!(context, config);
        let mut contract = StatusMessage::default();
        contract.set_status("hello".to_string());
        assert_eq!("hello".to_string(), contract.get_status("bob.near".to_string()).unwrap());
    }

    #[test]
    fn get_nonexistent_message() {
        let context = get_context(vec![]);
        let config = Config::default();
        testing_env!(context, config);
        let contract = StatusMessage::default();
        assert_eq!(None, contract.get_status("francis.near".to_string()));
    }
}
