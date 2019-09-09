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
    last_block_number: u64
}

#[near_bindgen]
impl EthBridge {
    pub fn add_block_headers(&mut self, start: u64, block_headers: Vec<Vec<u8>>) {
        let mut best_block_number = 0u64;
        for i in start..(start + block_headers.len() as u64) {
            match rlp::decode::<BlockHeader>(block_headers[i as usize].as_slice()) {
                Ok(block_header) => {
                    assert!(block_header.number == self.last_block_number + 1);
                    match self.block_hash(self.last_block_number) {
                        Some(hash) => assert_eq!(block_header.parent_hash, hash.into()),
                        None => {},
                    }
                    self.block_hashes.insert(i, block_header.hash.into());
                    best_block_number = block_header.number;
                },
                Err(_e) => {
                    panic!();
                },
            }
        }
        self.last_block_number = best_block_number;
    }

    pub fn block_hash(&self, index: u64) -> Option<[u8; 32]> {
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
