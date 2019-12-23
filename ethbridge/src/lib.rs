extern crate crypto;
extern crate rlp;

use std::collections::HashMap;
use ethereum_types::BigEndianHash;
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

    pub fn add_block_headers(&mut self, start: u64, block_headers: Vec<Vec<u8>>) {
        let mut prev_hash: Option<H256> = self.block_hashes.get(&(start - 1)).cloned();
        for access_index in 0..block_headers.len() {
            let block_number = start + access_index as u64;
            let header = rlp::decode::<BlockHeader>(block_headers[access_index].as_slice()).unwrap();
            assert_eq!(header.number(), block_number);

            // Check prev block compatibility
            match prev_hash {
                Some(hash) => {
                    assert_eq!(header.parent_hash(), hash);
                },
                None => {
                    // Only can happen on very first blocks
                },
            }

            self.block_hashes.insert(block_number, header.hash().unwrap());
            prev_hash = header.hash();

            // Update self.last_block_number only on latest iteration
            if access_index == block_headers.len() - 1 {
                // Check longest chain rule
                assert!(header.number() > self.last_block_number);
                self.last_block_number = header.number();
            }
        }
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

    pub fn difficulty(target: ethereum_types::U256) -> ethereum_types::U256 {
        if target <= ethereum_types::U256::one() {
            ethereum_types::U256::max_value()
        } else {
            ((ethereum_types::U256::from(1) << 255) / target) << 1
        }
    }

    pub fn verifyHeader(
        &self,
        block_header: Vec<u8>,
        nonce: H64,
        nodes: Vec<NodeWithMerkleProof>,
    ) -> bool {
        let header = rlp::decode::<BlockHeader>(block_header.as_slice()).unwrap();
        let (n,m) = Self::hashimoto_merkle(
            self,
            header.hash().unwrap(),
            nonce,
            header.number(),
            nodes,
        );

        ethereum_types::U256::from((n.0).0) < ethash::cross_boundary(header.difficulty().0)
        // TODO: To be continued... see YellowPaper formula (50) in section 4.3.4
    }

    pub fn hashimoto_merkle(
        &self,
        header_hash: H256,
        nonce: H64,
        block_number: u64,
        nodes: Vec<NodeWithMerkleProof>,
    ) -> (H256, H256) {
        let pair = ethash::hashimoto(
            header_hash.0,
            nonce.0,
            ethash::get_full_size(block_number as usize),
            |i| {
                let node = nodes.iter().find(|&p| p.proof.index == i as u64 || p.proof.index + 1 == i as u64).unwrap();
                assert_eq!(node.apply_merkle_proof(), self.dags_merkle_roots[block_number as usize / 30000]);
                node.dag_nodes[i - node.proof.index as usize].0
            }
        );
        (H256(pair.0), H256(pair.1))
    }
}
