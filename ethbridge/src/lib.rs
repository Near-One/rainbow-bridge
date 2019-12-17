extern crate crypto;
extern crate rlp;

use std::collections::HashMap;
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::{near_bindgen};
use self::crypto::digest::Digest;
use self::crypto::sha3::Sha3;
use std::io::{Error, Read, Write};
use ethash;

mod header;
use header::{BlockHeader};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct H64(pub ethereum_types::H64);

#[near_bindgen]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct H128(pub ethereum_types::H128);

#[near_bindgen]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct H256(pub ethereum_types::H256);

#[near_bindgen]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct H512(pub ethereum_types::H512);

impl BorshDeserialize for H64 {
    #[inline]
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(H64(ethereum_types::H64(<[u8; 8]>::deserialize(reader)?)))
    }
}

impl BorshSerialize for H64 {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (self.0).0.serialize(writer)
    }
}

impl BorshDeserialize for H128 {
    #[inline]
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(H128(ethereum_types::H128(<[u8; 16]>::deserialize(reader)?)))
    }
}

impl BorshSerialize for H128 {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (self.0).0.serialize(writer)
    }
}

impl BorshDeserialize for H256 {
    #[inline]
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(H256(ethereum_types::H256(<[u8; 32]>::deserialize(reader)?)))
    }
}

impl BorshSerialize for H256 {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (self.0).0.serialize(writer)
    }
}

impl BorshDeserialize for H512 {
    #[inline]
    fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(H512(ethereum_types::H512(<[u8; 64]>::deserialize(reader)?)))
    }
}

impl BorshSerialize for H512 {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (self.0).0.serialize(writer)
    }
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct EthBridge {
    dags_start_epoch: u64,
    dags_merke_roots: Vec<H128>,
    block_hashes: HashMap<u64, [u8; 32]>,
    last_block_number: u64,
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct MerkleProof {
    leafs: Vec<H128>,
    index: u64,
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct NodeWithMerkleProof {
    dag_node: H512,
    proof: MerkleProof,
}

impl NodeWithMerkleProof {

    fn truncate_to_h128(arr: &[u8]) -> H128 {
        let mut data: [u8; 16] = Default::default();
        H128(ethereum_types::H128(
            {data.copy_from_slice(&arr[arr.len()-16..]); data}
        ))
    }

    fn merge_h128(l: H128, r: H128) -> [u8; 32] {
        let mut data: [u8; 32] = Default::default();
        data[..16].copy_from_slice(&(l.0).0);
        data[16..].copy_from_slice(&(r.0).0);
        data
    }

    pub fn verify(&self, root: H128) -> bool {
        let mut leaf = Self::truncate_to_h128(&keccak256(&(self.dag_node.0).0)[16..]);
        for i in 0..self.proof.leafs.len() {
            if (self.proof.index & (1 << i)) == 0 {
                leaf = Self::truncate_to_h128(&keccak256(&Self::merge_h128(leaf, self.proof.leafs[i])));
            } else {
                leaf = Self::truncate_to_h128(&keccak256(&Self::merge_h128(self.proof.leafs[i], leaf)));
            }
        }
        (root.0).0 == (leaf.0).0
    }
}

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3::keccak256();
    hasher.input(data);

    let mut buffer = [0u8; 32];
    hasher.result(&mut buffer);
    buffer
}

#[near_bindgen]
impl EthBridge {

    const NUMBER_OF_FUTURE_BLOCKS: u64 = 10;

    pub fn init(&mut self, dags_start_epoch: u64, dags_merke_roots: Vec<H128>) {
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

    pub fn dag_merkle_root(&self, epoch: u64) -> H128 {
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

    pub fn hashimoto_merkle(
        &self,
        header_hash: H256,
        nonce: H64,
        full_size: usize,
        block_number: u64,
        nodes: Vec<NodeWithMerkleProof>,
    ) -> (H256, H256) {
        let ab = ethash::hashimoto(header_hash.0, nonce.0, full_size, |i| {
            let node = nodes.iter().find(|&p| p.proof.index == i as u64).unwrap();
            assert!(node.verify(self.dags_merke_roots[block_number as usize / 30000]));
            node.dag_node.0
        });
        (H256(ab.0), H256(ab.1))
    }
}
