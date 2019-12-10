extern crate crypto;
extern crate rlp;

use std::collections::HashMap;
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::{near_bindgen};
use self::crypto::digest::Digest;
use self::crypto::sha3::Sha3;

mod header;
use header::{BlockHeader};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct EthBridge {
    dags_start_epoch: u64,
    dags_merke_roots: Vec<[u8; 16]>,
    block_hashes: HashMap<u64, [u8; 32]>,
    last_block_number: u64,
}

fn keccak512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha3::keccak512();
    hasher.input(data);

    let mut buffer = [0u8; 64];
    hasher.result(&mut buffer);
    buffer
}

#[near_bindgen]
impl EthBridge {

    const NUMBER_OF_FUTURE_BLOCKS: u64 = 10;

    pub fn init(&mut self, dags_start_epoch: u64, dags_merke_roots: Vec<[u8; 16]>) {
        assert!(self.dags_merke_roots.len() == 0 && dags_merke_roots.len() > 0);
        self.dags_start_epoch = dags_start_epoch;
        self.dags_merke_roots = dags_merke_roots;
    }

    pub fn add_block_headers(&mut self, start: u64, block_headers: Vec<Vec<u8>>) {
        let mut prev_hash: Option<[u8; 32]> = self.block_hashes.get(&(start - 1)).cloned();
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

    pub fn dag_merkle_root(&self, epoch: u64) -> [u8; 16] {
        self.dags_merke_roots[(&epoch - self.dags_start_epoch) as usize]
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

    //

    pub const DATASET_BYTES_INIT: u64 = 1 << 30;
    pub const DATASET_BYTES_GROWTH: u64 = 1 << 23;
    pub const CACHE_BYTES_INIT: u64 = 1 << 24;
    pub const CACHE_BYTES_GROWTH: u64 = 1 << 17;

    pub const ETHASH_EPOCH_LENGTH: u64 = 30000;
    pub const ETHASH_CACHE_ROUNDS: u32 = 3;
    pub const ETHASH_MIX_BYTES: u32 = 128;
    pub const ETHASH_ACCESSES: u32 = 64;
    pub const ETHASH_DATASET_PARENTS: u32 = 256;
    pub const NODE_BYTES: u32 = 64;
    pub const NODE_WORDS: u32 = Self::NODE_BYTES / 4;
    pub const NODE_DWORDS: u32 = Self::NODE_WORDS / 2;
    pub const MIX_WORDS: u32 = Self::ETHASH_MIX_BYTES / 4;
    pub const MIX_NODES: u32 = Self::MIX_WORDS / Self::NODE_WORDS;
    pub const FNV_PRIME: u32 = 0x01000193;
    
    fn slice_to_array_4<T: Copy>(a: &[T]) -> [T; 4] {
        return [
            a[0], a[1], a[2], a[3]
        ];
    }

    fn epoch(block_number: u64) -> u64 {
        block_number / Self::ETHASH_EPOCH_LENGTH
    }

    fn fnv_hash(x: u32, y: u32) -> u32 {
        return x.wrapping_mul(Self::FNV_PRIME) ^ y;
    }

    fn get_data_size(block_number: u64) -> u32 {
        let mut sz: u64 = Self::DATASET_BYTES_INIT + Self::DATASET_BYTES_GROWTH * (block_number / Self::ETHASH_EPOCH_LENGTH);
        sz = sz - Self::ETHASH_MIX_BYTES as u64;
        while !primal::is_prime(sz / Self::ETHASH_MIX_BYTES as u64) {
            sz = sz - 2 * Self::ETHASH_MIX_BYTES as u64;
        }
        sz as u32
    }

    fn verify_dag_item(
        _node_index: u32,
        _node_data: [u8; 64],
        _merkleProof: &Vec<[u8; 16]>,
    ) -> bool {
        return true;
    }

    fn check_pow(header: BlockHeader, nonce: u64, dag_nodes: Vec<[u8; 64]>, merkleProofs: Vec<Vec<[u8; 16]>>) {
        
        // Pack header hash and nonce together into 40 bytes
        let mut hash_and_nonce = [0u8; 40];
        hash_and_nonce[..32].clone_from_slice(&header.hash().unwrap());
        hash_and_nonce[32..].clone_from_slice(&nonce.to_le_bytes());

        // Compute sha3-512 hash of (header hash + nonce)
        let hash = keccak512(&hash_and_nonce);

        // Replicate hash across mix
        let mut mix = [0u8; 128];
        mix[..64].clone_from_slice(&hash);
        mix[64..].clone_from_slice(&hash);

        let full_size = Self::get_data_size(header.number());
        let page_size = 4 * Self::MIX_WORDS;
	    let num_full_pages = (full_size / page_size) as u32;
        let first_val = u32::from_le_bytes(Self::slice_to_array_4(&hash[..4]));

        // https://github.com/paritytech/parity-ethereum/blob/master/ethash/src/compute.rs#L232
        for access_index in 0..Self::ETHASH_ACCESSES as u32 {
            let index = {
                let mix_index = (access_index % Self::MIX_WORDS) as usize;
                let mix_bytes = u32::from_le_bytes(Self::slice_to_array_4(&mix[mix_index..mix_index+4]));
                Self::fnv_hash(first_val ^ access_index, mix_bytes) % num_full_pages
            };

            for i in 0..Self::MIX_NODES {

                let node = dag_nodes[(access_index * Self::MIX_NODES + i) as usize];

                assert!(Self::verify_dag_item(
                    (index * Self::MIX_NODES + i) as u32,
                    node,
                    &merkleProofs[(access_index * Self::MIX_NODES + i) as usize]
                ));

                for j in 0..32 {
                    mix[j*4..j*4+4].clone_from_slice(
                        &Self::fnv_hash(
                            u32::from_le_bytes(Self::slice_to_array_4(&mix[j*4..j*4+4])),
                            u32::from_le_bytes(Self::slice_to_array_4(&node[j*4..j*4+4]))
                        ).to_le_bytes()
                    );
                }
            }
        }

        println!("{:?}", mix[1]);
    }
}
