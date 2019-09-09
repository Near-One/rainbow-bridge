extern crate crypto;

use rlp::{Rlp, DecoderError, Decodable};
use ethereum_types::{H256, U256, Address, Bloom};
use self::crypto::digest::Digest;
use self::crypto::sha3::Sha3;

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub parent_hash: H256,
    pub timestamp: u64,
    pub number: u64,
    pub author: Address,

    pub transactions_root: H256,
    pub uncles_hash: H256,
    pub extra_data: Vec<u8>,

    pub state_root: H256,
    pub receipts_root: H256,
    pub log_bloom: Bloom,
    pub gas_used: U256,
    pub gas_limit: U256,

    pub difficulty: U256,

    pub hash: H256,
}

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3::keccak256();
    hasher.input(data);

    let mut buffer = [0u8; 32];
    hasher.result(&mut buffer);
    buffer
}

impl Decodable for BlockHeader {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let blockheader = BlockHeader {
            parent_hash: rlp.val_at(0)?,
            uncles_hash: rlp.val_at(1)?,
            author: rlp.val_at(2)?,
            state_root: rlp.val_at(3)?,
            transactions_root: rlp.val_at(4)?,
            receipts_root: rlp.val_at(5)?,
            log_bloom: rlp.val_at(6)?,
            difficulty: rlp.val_at(7)?,
            number: rlp.val_at(8)?,
            gas_limit: rlp.val_at(9)?,
            gas_used: rlp.val_at(10)?,
            timestamp: rlp.val_at(11)?,
            extra_data: rlp.val_at(12)?,
            hash: keccak256(rlp.as_raw()).into(),
        };

        Ok(blockheader)
    }
}
