use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen};
use std::collections::HashMap;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod tests;

// add the following attributes to prepare your code for serialization and invocation on the blockchain
// More built-in Rust attributes here: https://doc.rust-lang.org/reference/attributes.html#built-in-attributes-index
#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct BtcClientContract {
    // See more data types at https://doc.rust-lang.org/book/ch03-02-data-types.html
    val: i8, // i8 is signed. unsigned integers are also available: u8, u16, u32, u64, u128

    most_recent_block_hash: String,
    blocks: HashMap<String, Block>,
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Block {
    header: Header,
    height: u128
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Header {
    prev_block_hash: String,
    merkle_root_hash: String,
    version: u32,
    time: u32,
    n_bits: u32,
    nonce: u32
}

#[near_bindgen]
impl BtcClientContract {    
    /** 
     * Accept a block header and verifies proof of work.
     */
    pub fn accept_header(block_header: String) {
        println!("Received block header {}", block_header);

        // // Convert block_header into hash
        // block_header_hash = sha256(sha256(block_header))

        // if (verify_header(block_header_hash)) {
        //     let prev_block_hash = get_prev_block_hash(block_header)
        //     let header = Header { prev_block_hash: prev_block_hash, }
        //     let new_block = Block { header: header }
        //     self.blocks[block_header_hash] = new_block
        // }
    }

    /** 
     * Method that accepts hash of the header and returns true if the header was 
     * verified and accepted. False otherwise.
     */
    pub fn verify_header(block_header_hash: String) -> bool {
        return true
    }

    /**
     * Accepts proof of a specific transaction hash and verifies that the header in the 
     * proof was verified, and then checks that the merkle path in the proof is correct.
     */
    pub fn verify_txn(txHash: String) {

    }

    /// Returns 8-bit signed integer of the counter value.
    ///
    /// This must match the type from our struct's 'val' defined above.
    ///
    /// Note, the parameter is `&self` (without being mutable) meaning it doesn't modify state.
    /// In the frontend (/src/main.js) this is added to the "viewMethods" array
    /// using near-shell we can call this by:
    ///
    /// ```bash
    /// near view counter.YOU.testnet get_num
    /// ```
    pub fn get_num(&self) -> i8 {
        return self.val;
    }

    /// Increment the counter.
    ///
    /// Note, the parameter is "&mut self" as this function modifies state.
    /// In the frontend (/src/main.js) this is added to the "changeMethods" array
    /// using near-shell we can call this by:
    ///
    /// ```bash
    /// near call counter.YOU.testnet increment --accountId donation.YOU.testnet
    /// ```
    pub fn increment(&mut self) {
        // note: adding one like this is an easy way to accidentally overflow
        // real smart contracts will want to have safety checks
        self.val += 1;
        let log_message = format!("Increased number to {}", self.val);
        env::log(log_message.as_bytes());
        after_counter_change();
    }

    /// Decrement (subtract from) the counter.
    ///
    /// In (/src/main.js) this is also added to the "changeMethods" array
    /// using near-shell we can call this by:
    ///
    /// ```bash
    /// near call counter.YOU.testnet decrement --accountId donation.YOU.testnet
    /// ```
    pub fn decrement(&mut self) {
        // note: subtracting one like this is an easy way to accidentally overflow
        // real smart contracts will want to have safety checks
        self.val -= 1;
        let log_message = format!("Decreased number to {}", self.val);
        env::log(log_message.as_bytes());
        after_counter_change();
    }

    /// Reset to zero.
    pub fn reset(&mut self) {
        self.val = 0;
        // Another way to log is to cast a string into bytes, hence "b" below:
        env::log(b"Reset counter to zero");
    }
}

// unlike the struct's functions above, this function cannot use attributes #[derive(…)] or #[near_bindgen]
// any attempts will throw helpful warnings upon 'cargo build'
// while this function cannot be invoked directly on the blockchain, it can be called from an invoked function
fn after_counter_change() {
    // show helpful warning that i8 (8-bit signed integer) will overflow above 127 or below -128
    env::log("Make sure you don't overflow, my friend.".as_bytes());
}
