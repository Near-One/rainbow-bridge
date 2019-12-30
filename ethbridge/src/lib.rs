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
    block_difficulties: HashMap<u64, U256>,
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

    //
    // Usually each next sequence should start from the last added block:
    // [1]-[2]-[3]-[4]
    //             [4]-[5]-[6]-[7]
    //
    // In case of reorg next sequence can start from the last same block:
    // [1]-[2]-[3]-[4a]
    //         [3]-[4b]-[5]-[6]
    //
    pub fn add_block_headers(
        &mut self,
        block_headers: Vec<Vec<u8>>,
        dag_nodes: Vec<Vec<DoubleNodeWithMerkleProof>>,
    ) {
        let mut prev = rlp::decode::<BlockHeader>(block_headers[0].as_slice()).unwrap();

        let very_first_blocks = self.last_block_number == 0;
        if very_first_blocks {
            // Submit very first block, can trust relayer
            self.block_hashes.insert(prev.number, prev.hash.unwrap());
            self.last_block_number = prev.number;
        } else {
            // Check first block hash equals to submitted one
            assert_eq!(prev.hash.unwrap(), self.block_hashes[&prev.number]);
        }

        let mut origin_total_difficulty = U256(ethereum_types::U256::from(0));
        let mut branch_total_difficulty = U256(ethereum_types::U256::from(0));

        // Check validity of all the following blocks
        for i in 1..block_headers.len() {
            let header = rlp::decode::<BlockHeader>(block_headers[i].as_slice()).unwrap();
            assert!(Self::verify_header(
                &self,
                &header,
                &prev,
                &dag_nodes[i]
            ));

            // Compute new chain total difficulty
            branch_total_difficulty = U256(
                branch_total_difficulty.0 + header.difficulty.0
            );
            if header.number <= self.last_block_number {
                // Compute old chain total difficulty if reorg
                origin_total_difficulty = U256(
                    origin_total_difficulty.0 + self.block_difficulties[&header.number].0
                );
            }

            self.block_hashes.insert(header.number, header.hash.unwrap());
            self.block_difficulties.insert(header.number, header.difficulty);
            prev = header;
        }

        if !very_first_blocks {
            // Ensure the longest chain rule: https://ethereum.stackexchange.com/a/13750/3032
            // https://github.com/ethereum/go-ethereum/blob/525116dbff916825463931361f75e75e955c12e2/core/blockchain.go#L863
            assert!(
                branch_total_difficulty.0 > origin_total_difficulty.0 ||
                (
                    branch_total_difficulty.0 == origin_total_difficulty.0 &&
                    (prev.hash.unwrap().0).0[0]%2 == 0 // hash is good enough random for us
                )
            );
        }
        self.last_block_number = prev.number;
    }

    pub fn verify_header(
        &self,
        header: &BlockHeader,
        prev: &BlockHeader,
        dag_nodes: &Vec<DoubleNodeWithMerkleProof>,
    ) -> bool {
        let (_mix_hash, result) = Self::hashimoto_merkle(
            self,
            header.partial_hash.unwrap(),
            header.nonce,
            header.number,
            dag_nodes.to_vec(),
        );

        //
        // See YellowPaper formula (50) in section 4.3.4
        // 1. Simplified difficulty check to conform adjusting difficulty bomb
        // 2. Added condition: header.parent_hash() == prev.hash()
        //
        ethereum_types::U256::from((result.0).0) < ethash::cross_boundary(header.difficulty.0)
        && header.difficulty.0 < header.difficulty.0 * ethereum_types::U256::from(101) / ethereum_types::U256::from(100)
        && header.difficulty.0 > header.difficulty.0 * ethereum_types::U256::from(99) / ethereum_types::U256::from(100)
        && header.gas_used.0 <= header.gas_limit.0
        && header.gas_limit.0 < prev.gas_limit.0 * ethereum_types::U256::from(1025) / ethereum_types::U256::from(1024)
        && header.gas_limit.0 > prev.gas_limit.0 * ethereum_types::U256::from(1023) / ethereum_types::U256::from(1024)
        && header.gas_limit.0 >= ethereum_types::U256::from(5000)
        && header.timestamp > prev.timestamp
        && header.number == prev.number + 1
        && header.parent_hash == prev.hash.unwrap()
    }

    pub fn hashimoto_merkle(
        &self,
        header_hash: H256,
        nonce: H64,
        block_number: u64,
        nodes: Vec<DoubleNodeWithMerkleProof>,
    ) -> (H256, H256) {
        // Boxed index since ethash::hashimoto gets Fn, but not FnMut
        let index = std::cell::RefCell::new(0);
        let merkle_root = self.dag_merkle_root((block_number as usize / 30000) as u64);
        let pair = ethash::hashimoto(
            header_hash.0,
            nonce.0,
            ethash::get_full_size(block_number as usize / 30000),
            |offset| {
                let idx = *index.borrow_mut();
                *index.borrow_mut() += 1;

                // Each two nodes are packed into single 128 bytes with Merkle proof
                let node = &nodes[idx / 2];
                if idx % 2 == 0 {
                    // Divide by 2 to adjust offset for 64-byte words instead of 128-byte
                    assert_eq!(merkle_root, node.apply_merkle_proof((offset / 2) as u64));
                };

                // Reverse each 32 bytes for ETHASH compatibility
                let mut data = (node.dag_nodes[idx % 2].0).0;
                data[..32].reverse();
                data[32..].reverse();
                ethereum_types::H512(data)
            }
        );

        (H256(pair.0), H256(pair.1))
    }
}
