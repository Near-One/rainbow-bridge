use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen};
use std::collections::HashMap;
use std::str;
use byteorder::{ByteOrder, LittleEndian};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod tests;

// add the following attributes to prepare your code for serialization and invocation on the blockchain
// More built-in Rust attributes here: https://doc.rust-lang.org/reference/attributes.html#built-in-attributes-index
#[near_bindgen]
pub struct BtcClientContract {
    most_recent_block_hash: HashStr,
    blocks: HashMap<HashStr, BlockHeader>, // Map of block hash to block header.
}

#[derive(Clone)]
pub struct BlockHeader {
    block_hash: HashStr,
    version: u32,
    prev_block_hash: HashStr,
    merkle_root_hash: HashStr,
    time: u32,
    n_bits: u32,
    nonce: u32
}

#[derive(Hash, Clone, Eq, PartialEq)]
struct HashStr {
    value: String
}

#[near_bindgen]
impl BtcClientContract {    
    /** 
     * Accept a block header and verifies proof of work.
     */
    pub fn accept_header(
        &mut self,
        block_header: BlockHeader,
    ) {
        if (Self::verify_header(self, block_header.clone())) {
            println!("Added new block header. Block hash: {}", block_header.block_hash.value);
            self.blocks.insert(block_header.block_hash.clone(), block_header.clone());
            self.most_recent_block_hash = block_header.block_hash.clone();
        }
    }

    /** 
     * Method that accepts hash of the header and returns true if the header was 
     * verified and accepted. False otherwise.
     */
    pub fn verify_header(
        &self,
        block_header: BlockHeader
    ) -> bool {
        let calculated_block_hash = Self::calculate_block_hash(block_header.clone());
        if (calculated_block_hash != block_header.block_hash.value) {
            return false;
        }

        // TODO verify difficulty.
        // TODO verify that we're within 6 confirmations.

        return true;
    }

    /**
     * Accepts proof of a specific transaction hash and verifies that the header in the 
     * proof was verified, and then checks that the merkle path in the proof is correct.
     */
    pub fn verify_txn(txHash: String) {

    } 
}

impl BtcClientContract { 
    fn calculate_block_hash(block_header: BlockHeader) -> String {
        let version_hex = Self::get_hex(block_header.version);
        let prev_hash_hex = Self::little_endian(block_header.prev_block_hash.value);
        let merkle_root_hex = Self::little_endian(block_header.merkle_root_hash.value);
        let time_hex = Self::get_hex(block_header.time);
        let nbits_hex = Self::get_hex(block_header.n_bits);
        let nonce_hex = Self::get_hex(block_header.nonce);

        // println!("{}", version_hex);
        // println!("{}", prev_hash_hex);
        // println!("{}", merkle_root_hex);
        // println!("{}", time_hex);
        // println!("{}", nbits_hex);
        // println!("{}", nonce_hex);

        let aggregate_hex_str = [version_hex, prev_hash_hex, merkle_root_hex, time_hex, nbits_hex, nonce_hex].concat();
        let bytes = hex::decode(aggregate_hex_str).unwrap();

        return Self::little_endian(hex::encode(env::sha256(&env::sha256(&bytes))));
    }

    fn get_hex(n: u32) -> String {
        let mut bytes: [u8; 4] = [0; 4];
        LittleEndian::write_u32_into(&[n], &mut bytes);
        return hex::encode(bytes);
    }

    fn little_endian(hex_str: String) -> String {
        let mut vec = hex::decode(hex_str).unwrap();
        vec.reverse();
        return hex::encode(&vec);
    }
}
