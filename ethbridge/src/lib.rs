extern crate crypto;
extern crate rlp;

use std::collections::HashMap;
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::{near_bindgen};
use ethash;

pub mod header;
use header::*;

pub mod types;
use types::*;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct EthBridge {
    dags_start_epoch: u64,
    dags_merkle_roots: Vec<H128>,
    block_hashes: HashMap<u64, H256>,
    last_block_number: u64,
}

#[near_bindgen]
impl EthBridge {

    const NUMBER_OF_FUTURE_BLOCKS: u64 = 10;

    pub fn init(&mut self, dags_start_epoch: u64, dags_merkle_roots: Vec<H128>) {
        assert!(self.dags_merkle_roots.len() == 0 && dags_merkle_roots.len() > 0);
        self.dags_start_epoch = dags_start_epoch;
        self.dags_merkle_roots = dags_merkle_roots;
    }

    pub fn dag_merkle_root(&self, epoch: u64) -> H128 {
        self.dags_merkle_roots[(&epoch - self.dags_start_epoch) as usize]
    }

    pub fn block_hash_unsafe(&self, index: u64) -> Option<H256> {
        self.block_hashes.get(&index).cloned()
    }

    pub fn block_hash(&self, index: u64) -> Option<H256> {
        if index + EthBridge::NUMBER_OF_FUTURE_BLOCKS > self.last_block_number {
            return Option::None;
        }
        self.block_hashes.get(&index).cloned()
    }

    pub fn add_block_headers(
        &mut self,
        block_headers: Vec<Vec<u8>>,
        dag_nodes: Vec<Vec<DoubleNodeWithMerkleProof>>,
    ) {
        let mut prev = rlp::decode::<BlockHeader>(block_headers[0].as_slice()).unwrap();

        if self.last_block_number == 0 {
            // Submit very first block, can trust relayer
            self.block_hashes.insert(prev.number(), prev.hash().unwrap());
            self.last_block_number = prev.number();
        } else {
            // Check first block hash equals submitted one
            assert_eq!(prev.hash().unwrap(), self.block_hashes[&prev.number()]);
        }

        // Check validity of all the following blocks
        for i in 1..block_headers.len() {
            let header = rlp::decode::<BlockHeader>(block_headers[i].as_slice()).unwrap();
            assert!(Self::verify_header(
                &self,
                &header,
                &prev,
                &dag_nodes[i]
            ));

            self.block_hashes.insert(header.number(), header.hash().unwrap());
            prev = header;
        }

        // Ensure submitted sequence is not shorter than previous one
        assert!(prev.number() >= self.last_block_number);
        self.last_block_number = prev.number();
    }

    fn verify_header(
        &self,
        header: &BlockHeader,
        prev: &BlockHeader,
        dag_nodes: &Vec<DoubleNodeWithMerkleProof>,
    ) -> bool {
        let (_mix_hash, result) = Self::hashimoto_merkle(
            self,
            header.partial_hash().unwrap(),
            header.nonce(),
            header.number(),
            dag_nodes.to_vec(),
        );

        //
        // See YellowPaper formula (50) in section 4.3.4
        // 1. Simplified difficulty check to conform adjusting difficulty bomb
        // 2. Added condition: header.parent_hash() == prev.hash()
        //
        ethereum_types::U256::from((result.0).0) < ethash::cross_boundary(header.difficulty().0)
        && header.difficulty().0 < header.difficulty().0 * ethereum_types::U256::from(101) / ethereum_types::U256::from(100)
        && header.difficulty().0 > header.difficulty().0 * ethereum_types::U256::from(99) / ethereum_types::U256::from(100)
        && header.gas_used().0 <= header.gas_limit().0
        && header.gas_limit().0 < prev.gas_limit().0 * ethereum_types::U256::from(1025) / ethereum_types::U256::from(1024)
        && header.gas_limit().0 > prev.gas_limit().0 * ethereum_types::U256::from(1023) / ethereum_types::U256::from(1024)
        && header.gas_limit().0 > ethereum_types::U256::from(5000)
        && header.timestamp() > prev.timestamp()
        && header.number() == prev.number() + 1
        && header.parent_hash() == prev.hash().unwrap()
    }

    pub fn hashimoto_merkle(
        &self,
        header_hash: H256,
        nonce: H64,
        block_number: u64,
        nodes: Vec<DoubleNodeWithMerkleProof>,
    ) -> (H256, H256) {
        dbg!(header_hash);
        dbg!(nonce);
        dbg!(block_number);
        let index = std::cell::RefCell::new(0);
        let merkle_root = self.dag_merkle_root((block_number as usize / 30000) as u64);
        let pair = ethash::hashimoto(
            header_hash.0,
            nonce.0,
            ethash::get_full_size(block_number as usize / 30000),
            |offset| {
                dbg!(offset);
                let idx = *index.borrow_mut();
                *index.borrow_mut() += 1;

                let node = &nodes[idx / 2];
                if idx % 2 == 0 {
                    assert_eq!(node.apply_merkle_proof((offset / 2) as u64), merkle_root);
                    dbg!("OK");
                }
                node.dag_nodes[idx % 2].0
            }
        );
        (H256(pair.0), H256(pair.1))
    }
}
