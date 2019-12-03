#![feature(const_vec_new)]

extern crate crypto;
extern crate rlp;

use std::collections::HashMap;
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::{near_bindgen};

mod header;
use header::{BlockHeader};

#[cfg(not(feature = "env_test"))]
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
        let mut prev_hash: Option<[u8; 32]> = self.block_hashes.get(&(start - 1)).cloned();
        for i in 0..block_headers.len() {
            let block_number = start + i as u64;
            let header = rlp::decode::<BlockHeader>(block_headers[i].as_slice()).unwrap();
            
            // Check prev block compatibility
            assert_eq!(header.number(), block_number);
            match prev_hash {
                Some(hash) => {
                    assert_eq!(header.parent_hash(), hash);
                },
                None => {
                    // Only can happen on first iteration
                },
            }

            self.block_hashes.insert(block_number, header.hash().unwrap());
            prev_hash = header.hash();

            // Update self.last_block_number only on latest iteration
            if i == block_headers.len() - 1 {
                // Check longest chain rule
                assert!(header.number() > self.last_block_number);
                self.last_block_number = header.number();
            }
        }
    }

    pub fn block_hash_unsafe(&self, index: u64) -> Option<[u8; 32]> {
        self.block_hashes.get(&index).cloned()
    }

    pub fn block_hash(&self, index: u64) -> Option<[u8; 32]> {
        if index + EthBridge::NUMBER_OF_FUTURE_BLOCKS > self.last_block_number {
            return Option::None;
        }
        self.block_hashes.get(&index).cloned()
    }
}
