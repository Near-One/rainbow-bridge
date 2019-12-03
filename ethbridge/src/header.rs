extern crate crypto;

use rlp::{Rlp, DecoderError, Decodable};
use ethereum_types::{H256, U256, Address, Bloom};
use self::crypto::digest::Digest;
use self::crypto::sha3::Sha3;

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

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3::keccak256();
    hasher.input(data);

    let mut buffer = [0u8; 32];
    hasher.result(&mut buffer);
    buffer
}

impl BlockHeader {
    pub fn parent_hash(&self) -> [u8; 32] {
        self.parent_hash.into()
    }

    pub fn number(&self) -> u64 {
        self.number.into()
    }

    pub fn hash(&self) -> Option<[u8; 32]> {
        self.hash.map(|h| h.into())
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
