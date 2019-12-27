use rlp::{Rlp, RlpStream, DecoderError as RlpDecoderError, Decodable as RlpDecodable, Encodable as RlpEncodable};
use ethereum_types;
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::{near_bindgen};

use crate::types::*;

#[near_bindgen]
#[derive(Default, Clone, BorshDeserialize, BorshSerialize)]
pub struct DoubleNodeWithMerkleProof {
    pub dag_nodes: Vec<H512>, // [H512; 2]
    pub proof: Vec<H128>,
}

impl DoubleNodeWithMerkleProof {
    fn truncate_to_h128(arr: H256) -> H128 {
        let mut data = [0u8; 16];
        data.copy_from_slice(&(arr.0).0[16..]);
        H128(ethereum_types::H128(data))
    }

    fn hash_h128(l: H128, r: H128) -> H128 {
        let mut data = [0u8; 64];
        data[16..32].copy_from_slice(&(l.0).0);
        data[48..64].copy_from_slice(&(r.0).0);
        Self::truncate_to_h128(sha256(&data))
    }

    pub fn apply_merkle_proof(&self, index: u64) -> H128 {
        let mut data = [0u8; 128];
        data[..64].copy_from_slice(&(self.dag_nodes[0].0).0);
        data[64..].copy_from_slice(&(self.dag_nodes[1].0).0);
        // for i in (0..128).step_by(32) {
        //     data[i..i+32].reverse();
        // }

        let mut leaf = Self::truncate_to_h128(sha256(&data));

        for i in 0..self.proof.len() {
            if (index & (1 << i)) != 0 {
                leaf = Self::hash_h128(leaf, self.proof[i]);
            } else {
                leaf = Self::hash_h128(self.proof[i], leaf);
            }
        }
        leaf
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
    difficulty: ethereum_types::U256,
    number: u64,
    gas_limit: ethereum_types::U256,
    gas_used: ethereum_types::U256,
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

    pub fn difficulty(&self) -> U256 {
        U256(self.difficulty)
    }

    pub fn gas_used(&self) -> U256 {
        U256(self.gas_used)
    }

    pub fn gas_limit(&self) -> U256 {
        U256(self.gas_limit)
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn hash(&self) -> Option<H256> {
        self.hash.map(|h| h.into())
    }

    pub fn extra_data(&self) -> H256 {
        let mut data = [0u8; 32];
        data.copy_from_slice(self.extra_data.as_slice());
        H256(ethereum_types::H256(data))
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

impl RlpEncodable for BlockHeader {
    fn rlp_append(&self, s: &mut RlpStream) {
        self.stream_rlp(s, false);
    }
}

impl RlpDecodable for BlockHeader {
    fn decode(rlp: &Rlp) -> Result<Self, RlpDecoderError> {
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
