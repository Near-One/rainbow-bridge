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

// Merkle proof for a single transaction.
pub struct MerkleProof {
    tx_hash: HashStr, // transaction hash that the Merkle proof is computed for.
    tx_index: u32, // index of the transaction among all transactions.
    siblings: Vec<HashStr> // sibling hashes of txId which comprise the Merkle proof.
}

#[derive(Default, Hash, Clone, Eq, PartialEq)]
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
        if (Self::verify_header(self, &block_header)) {
            println!("Added new block header. Block hash: {}", block_header.block_hash.value);
            self.blocks.insert(block_header.block_hash.clone(), block_header.clone());
            self.most_recent_block_hash = block_header.block_hash;
        }
    }

    /** 
     * Method that accepts hash of the header and returns true if the header was 
     * verified and accepted. False otherwise.
     */
    pub fn verify_header(
        &self,
        block_header: &BlockHeader
    ) -> bool {
        let nbits_hex = Self::get_hex(block_header.n_bits);
        let calculated_block_hash = Self::calculate_block_hash(&block_header);

        if (calculated_block_hash != block_header.block_hash.value) {
            return false;
        }
        // TODO verify difficulty.
        if (Self::within6Confirms(self, block_header.clone())) {
            return false
        }
        
        // Verify the hash is smaller than the target.
        if (calculated_block_hash > nbits_hex) {
            return false;
        }
        return true;
    }

    fn within6Confirms(
        &self,
        block_header: BlockHeader
    ) -> bool {
        let mut block_hash = self.most_recent_block_hash.clone();
        let mut i = 0;
        while (i < 6) {
            if (block_header.block_hash.clone() == block_hash) {
                return true;
            }
            block_hash = block_header.prev_block_hash.clone();
            i += 1;
        }
        return false;
    }

    /**
     * Accepts proof of a specific transaction hash and verifies that the header in the 
     * proof was verified, and then checks that the merkle path in the proof is correct.
     *
     * Returns true if the tx is in the block given by 'txBlockHash' and the block is in Bitcoin's main chain.
     * Returns false if the tx is exactly 64 bytes long (to guard against a Merkle tree collision) or fails verification.
     * 
     * tx_index is the index of the tx within the block
     * sibling are the merkle siblings of tx
     */
    pub fn verify_tx(
        &self,
        merkle_proof: &MerkleProof,
        block_hash: &String
    ) -> bool {
        let block_header = match self.blocks.get(&HashStr{ value: block_hash.to_string() }) {
            Some(block_header) => block_header,
            None => return false
        };

        if (block_header.merkle_root_hash != Self::calculate_merkle_root(merkle_proof)) {
            return false;
        }

        // TODO check for 6 confirmations.
        // TODO check for paid fee.
        // TODO check tx_hash is in the main chain, ie not a fork.

        return true;
    } 

}

impl BtcClientContract { 
    fn calculate_merkle_root(merkle_proof: &MerkleProof) -> HashStr {
        let mut result_hash = merkle_proof.tx_hash.clone();
        let mut index = merkle_proof.tx_index;

        for i in 0..merkle_proof.siblings.len() {
            let mut left_hash: HashStr = Default::default();
            let mut right_hash: HashStr = Default::default();

            if index % 2 == 0 {
                left_hash = result_hash;
                right_hash = merkle_proof.siblings[i].clone();
            } else {
                left_hash = merkle_proof.siblings[i].clone();
                right_hash = result_hash;
            }

            result_hash = Self::concat_hash(left_hash.clone(), right_hash.clone());
            // println!("index: {}\n result: {}\n left: {}\n right: {}\n", i.to_string(), result_hash.value, left_hash.value, right_hash.value);
            index = index / 2;
        }

        return result_hash;
    }

    fn concat_hash(a: HashStr, b: HashStr) -> HashStr {
        let ab = [Self::little_endian(&a.value), Self::little_endian(&b.value)].concat();
        return HashStr{ value: Self::double_sha256(&ab) };
    }


    fn calculate_block_hash(block_header: &BlockHeader) -> String {
        let version_hex = Self::get_hex(block_header.version);
        let prev_hash_hex = Self::little_endian(&block_header.prev_block_hash.value);
        let merkle_root_hex = Self::little_endian(&block_header.merkle_root_hash.value);
        let time_hex = Self::get_hex(block_header.time);
        let nbits_hex = Self::get_hex(block_header.n_bits);
        let nonce_hex = Self::get_hex(block_header.nonce);

        // println!("{}", version_hex);
        // println!("{}", prev_hash_hex);
        // println!("{}", merkle_root_hex);
        // println!("{}", time_hex);
        // println!("{}", nbits_hex);
        // println!("{}", nonce_hex);

        return Self::double_sha256(&[version_hex, prev_hash_hex, merkle_root_hex, time_hex, nbits_hex, nonce_hex].concat());
    }

    /**
     * Returns a double sha256 hash of a string in little endian format.
     */
    fn double_sha256(str: &String) -> String {
        let bytes = hex::decode(str).unwrap();
        return Self::little_endian(&hex::encode(env::sha256(&env::sha256(&bytes))));
    }

    /**
     * Returns a hex string of a number.
     */
    fn get_hex(n: u32) -> String {
        let mut bytes: [u8; 4] = [0; 4];
        LittleEndian::write_u32_into(&[n], &mut bytes);
        return hex::encode(bytes);
    }

    /**
     * Returns the little endian version of the given hex string.
     */
    fn little_endian(hex_str: &String) -> String {
        let mut vec = hex::decode(&hex_str).unwrap();
        vec.reverse();
        return hex::encode(&vec);
    }
}
