use rlp::{Rlp, RlpStream, DecoderError, Decodable, Encodable};
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use ethereum_types;
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::{near_bindgen};

use crate::types::*;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct MerkleProof {
    pub leafs: Vec<H128>,
    pub index: u64,
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct NodeWithMerkleProof {
    pub dag_node: H512,
    pub proof: MerkleProof,
}

impl NodeWithMerkleProof {
    fn truncate_to_h128(arr: H256) -> H128 {
        let mut data: [u8; 16] = Default::default();
        H128(ethereum_types::H128(
            {data.copy_from_slice(&(arr.0).0[(arr.0).0.len()-16..]); data}
        ))
    }

    fn merge_h128(l: H128, r: H128) -> [u8; 32] {
        let mut data: [u8; 32] = Default::default();
        data[..16].copy_from_slice(&(l.0).0);
        data[16..].copy_from_slice(&(r.0).0);
        data
    }

    pub fn verify(&self, root: H128) -> bool {
        let mut leaf = Self::truncate_to_h128(keccak256(&(self.dag_node.0).0));
        for i in 0..self.proof.leafs.len() {
            if (self.proof.index & (1 << i)) == 0 {
                leaf = Self::truncate_to_h128(keccak256(&Self::merge_h128(leaf, self.proof.leafs[i])));
            } else {
                leaf = Self::truncate_to_h128(keccak256(&Self::merge_h128(self.proof.leafs[i], leaf)));
            }
        }
        (root.0).0 == (leaf.0).0
    }
}

#[derive(Debug, Clone)]
pub struct BlockHeader {
    parent_hash: H256,
    uncles_hash: H256,
    author: Address,
    state_root: H256,
    transactions_root: H256,
    receipts_root: H256,
    log_bloom: Bloom,
    difficulty: U256,
    number: u64,
    gas_limit: U256,
    gas_used: U256,
    timestamp: u64,
    extra_data: Vec<u8>,

    hash: Option<H256>,
}

impl BlockHeader {
    pub fn parent_hash(&self) -> H256 {
        self.parent_hash.into()
    }

    pub fn number(&self) -> u64 {
        self.number.into()
    }

    pub fn hash(&self) -> Option<H256> {
        self.hash.map(|h| h.into())
    }

    fn stream_rlp(&self, stream: &mut RlpStream, with_hash: bool) {
        stream.begin_list(13 + if with_hash { 1 } else { 0 });

        stream.append(&self.parent_hash);
        stream.append(&self.uncles_hash);
        stream.append(&self.author);
        stream.append(&self.state_root);
        stream.append(&self.transactions_root);
        stream.append(&self.receipts_root);
        stream.append(&self.log_bloom);
        stream.append(&self.difficulty);
        stream.append(&self.number);
        stream.append(&self.gas_limit);
        stream.append(&self.gas_used);
        stream.append(&self.timestamp);
        stream.append(&self.extra_data);

        if with_hash {
            stream.append(&self.hash);
        }
    }
}

impl Encodable for BlockHeader {
    fn rlp_append(&self, s: &mut RlpStream) {
        self.stream_rlp(s, false);
    }
}

impl Decodable for BlockHeader {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(BlockHeader {
            parent_hash:        rlp.val_at(0)?,
            uncles_hash:        rlp.val_at(1)?,
            author:             rlp.val_at(2)?,
            state_root:         rlp.val_at(3)?,
            transactions_root:  rlp.val_at(4)?,
            receipts_root:      rlp.val_at(5)?,
            log_bloom:          rlp.val_at(6)?,
            difficulty:         rlp.val_at(7)?,
            number:             rlp.val_at(8)?,
            gas_limit:          rlp.val_at(9)?,
            gas_used:           rlp.val_at(10)?,
            timestamp:          rlp.val_at(11)?,
            extra_data:         rlp.val_at(12)?,
            hash:               Some(keccak256(rlp.as_raw()).into()),
        })
    }
}
