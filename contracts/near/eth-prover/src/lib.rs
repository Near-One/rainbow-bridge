use borsh::{BorshDeserialize, BorshSerialize};
use eth_types::*;
use near_sdk::{env, ext_contract, near_bindgen, PromiseOrValue};
use rlp::Rlp;

#[cfg(test)]
mod tests;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type AccountId = String;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct EthProver {
    bridge_smart_contract: AccountId,
}

fn assert_self() {
    assert_eq!(env::current_account_id(), env::predecessor_account_id());
}

/// Defines an interface to call EthProver back as a callback with the result from the
/// EthClient contract.
#[ext_contract(remote_self)]
pub trait RemoteSelf {
    #[result_serializer(borsh)]
    fn on_block_hash(&self, #[serializer(borsh)] expected_block_hash: H256) -> bool;
}

/// Defines an interface to call EthClient contract to get the safe block hash for a given block
/// number. It returns Some(hash) if the block hash is present in the safe canonical chain, or
/// None if the block number is not part of the canonical chain yet.
#[ext_contract(eth_client)]
pub trait RemoteEthClient {
    #[result_serializer(borsh)]
    fn block_hash_safe(&self, #[serializer(borsh)] index: u64) -> Option<H256>;
}

impl Default for EthProver {
    fn default() -> Self {
        env::panic(b"Not initialized yet.");
    }
}

#[near_bindgen]
impl EthProver {
    #[init]
    pub fn init(#[serializer(borsh)] bridge_smart_contract: AccountId) -> Self {
        assert!(
            env::state_read::<EthProver>().is_none(),
            "The contract is already initialized"
        );
        Self {
            bridge_smart_contract,
        }
    }

    fn extract_nibbles(a: Vec<u8>) -> Vec<u8> {
        a.iter().flat_map(|b| vec![b >> 4, b & 0x0F]).collect()
    }

    fn concat_nibbles(a: Vec<u8>) -> Vec<u8> {
        a.iter()
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .zip(a.iter().enumerate().filter(|(i, _)| i % 2 == 1))
            .map(|((_, x), (_, y))| (x << 4) | y)
            .collect()
    }

    /// Implementation of the callback when the EthClient returns data.
    /// This method can only be called by the EthProver contract itself (e.g. as callback).
    /// - `block_hash` is the actual data from the EthClient call
    /// - `expected_block_hash` is the block hash that we expect to be passed by us.
    #[result_serializer(borsh)]
    pub fn on_block_hash(
        &self,
        #[callback]
        #[serializer(borsh)]
        block_hash: Option<H256>,
        #[serializer(borsh)] expected_block_hash: H256,
    ) -> bool {
        assert_self();
        return block_hash == Some(expected_block_hash);
    }

    /// Externally visible method to verify that the given block hash is part of the safe canonical
    /// chain on the remote EthClient contract.
    /// Returns a promise.
    #[result_serializer(borsh)]
    pub fn assert_ethclient_hash(
        &self,
        #[serializer(borsh)] block_number: u64,
        #[serializer(borsh)] expected_block_hash: H256,
    ) -> PromiseOrValue<bool> {
        eth_client::block_hash_safe(
            block_number,
            &self.bridge_smart_contract,
            0,
            env::prepaid_gas() / 3,
        )
        .then(remote_self::on_block_hash(
            expected_block_hash,
            &env::current_account_id(),
            0,
            10000000000000,
        ))
        .into()
    }

    #[result_serializer(borsh)]
    pub fn verify_log_entry(
        &self,
        #[serializer(borsh)] log_index: u64,
        #[serializer(borsh)] log_entry_data: Vec<u8>,
        #[serializer(borsh)] receipt_index: u64,
        #[serializer(borsh)] receipt_data: Vec<u8>,
        #[serializer(borsh)] header_data: Vec<u8>,
        #[serializer(borsh)] proof: Vec<Vec<u8>>,
        #[serializer(borsh)] skip_bridge_call: bool,
    ) -> PromiseOrValue<bool> {
        let log_entry: LogEntry = rlp::decode(log_entry_data.as_slice()).unwrap();
        let receipt: Receipt = rlp::decode(receipt_data.as_slice()).unwrap();
        let header: BlockHeader = rlp::decode(header_data.as_slice()).unwrap();

        // Verify log_entry included in receipt
        assert_eq!(receipt.logs[log_index as usize], log_entry);

        // Verify receipt included into header
        let verification_result = Self::verify_trie_proof(
            header.receipts_root,
            rlp::encode(&receipt_index),
            proof,
            receipt_data,
        );
        if verification_result && skip_bridge_call {
            return PromiseOrValue::Value(true);
        } else if !verification_result {
            return PromiseOrValue::Value(false);
        }

        // Verify block header was in the bridge
        eth_client::block_hash_safe(
            header.number,
            &self.bridge_smart_contract,
            0,
            10000000000000,
        )
        .then(remote_self::on_block_hash(
            header.hash.unwrap(),
            &env::current_account_id(),
            0,
            env::prepaid_gas() / 2,
        ))
        .into()
    }

    /// Iterate the proof following the key.
    /// Return True if the value at the leaf is equal to the expected value.
    /// @param expected_root is the expected root of the current proof node.
    /// @param key is the key for which we are proving the value.
    /// @param proof is the proof the key nibbles as path.
    /// @param expected_value is the key's value expected to be stored in
    ///     the last node (leaf node) of the proof.
    ///
    /// Patricia Trie: https://github.com/ethereum/wiki/wiki/Patricia-Tree#example-trie
    /// Patricia Img:  https://ethereum.stackexchange.com/questions/268/ethereum-block-architecture/6413#6413
    ///
    /// Verification:  https://github.com/slockit/in3/wiki/Ethereum-Verification-and-MerkleProof#receipt-proof
    /// Article:       https://medium.com/@ouvrard.pierre.alain/merkle-proof-verification-for-ethereum-patricia-tree-48f29658eec
    /// Python impl:   https://gist.github.com/paouvrard/7bb947bf5de0fa0dc69d0d254d82252a
    /// JS impl:       https://github.com/slockit/in3/blob/master/src/util/merkleProof.ts
    ///
    fn verify_trie_proof(
        expected_root: H256,
        key: Vec<u8>,
        proof: Vec<Vec<u8>>,
        expected_value: Vec<u8>,
    ) -> bool {
        let mut actual_key = vec![];
        for el in key {
            if actual_key.len() + 1 == proof.len() {
                actual_key.push(el);
            } else {
                actual_key.push(el / 16);
                actual_key.push(el % 16);
            }
        }
        Self::_verify_trie_proof(expected_root, actual_key, proof, 0, 0, expected_value)
    }

    fn _verify_trie_proof(
        expected_root: H256,
        key: Vec<u8>,
        proof: Vec<Vec<u8>>,
        key_index: usize,
        proof_index: usize,
        expected_value: Vec<u8>,
    ) -> bool {
        let node = &proof[proof_index];
        let dec = Rlp::new(&node.as_slice());

        if key_index == 0 {
            // trie root is always a hash
            assert_eq!(near_keccak256(node), (expected_root.0).0);
        } else if node.len() < 32 {
            // if rlp < 32 bytes, then it is not hashed
            assert_eq!(dec.as_raw(), (expected_root.0).0);
        } else {
            assert_eq!(near_keccak256(node), (expected_root.0).0);
        }

        if dec.iter().count() == 17 {
            // branch node
            if key_index == key.len() {
                if dec
                    .at(dec.iter().count() - 1)
                    .unwrap()
                    .as_val::<Vec<u8>>()
                    .unwrap()
                    == expected_value
                {
                    // value stored in the branch
                    return true;
                }
            } else if key_index < key.len() {
                let new_expected_root = dec
                    .at(key[key_index] as usize)
                    .unwrap()
                    .as_val::<Vec<u8>>()
                    .unwrap();
                if new_expected_root.len() != 0 {
                    return Self::_verify_trie_proof(
                        new_expected_root.into(),
                        key,
                        proof,
                        key_index + 1,
                        proof_index + 1,
                        expected_value,
                    );
                }
            } else {
                panic!("This should not be reached if the proof has the correct format");
            }
        } else if dec.iter().count() == 2 {
            // leaf or extension node
            // get prefix and optional nibble from the first byte
            let nibbles = Self::extract_nibbles(dec.at(0).unwrap().as_val::<Vec<u8>>().unwrap());
            let (prefix, nibble) = (nibbles[0], nibbles[1]);

            if prefix == 2 {
                // even leaf node
                let key_end = &nibbles[2..];
                if Self::concat_nibbles(key_end.to_vec()) == &key[key_index..]
                    && expected_value == dec.at(1).unwrap().as_val::<Vec<u8>>().unwrap()
                {
                    return true;
                }
            } else if prefix == 3 {
                // odd leaf node
                let key_end = &nibbles[2..];
                if nibble == key[key_index]
                    && Self::concat_nibbles(key_end.to_vec()) == &key[key_index + 1..]
                    && expected_value == dec.at(1).unwrap().as_val::<Vec<u8>>().unwrap()
                {
                    return true;
                }
            } else if prefix == 0 {
                // even extension node
                let shared_nibbles = &nibbles[2..];
                let extension_length = shared_nibbles.len();
                if Self::concat_nibbles(shared_nibbles.to_vec())
                    == &key[key_index..key_index + extension_length]
                {
                    let new_expected_root = dec.at(1).unwrap().as_val::<Vec<u8>>().unwrap();
                    return Self::_verify_trie_proof(
                        new_expected_root.into(),
                        key,
                        proof,
                        key_index + extension_length,
                        proof_index + 1,
                        expected_value,
                    );
                }
            } else if prefix == 1 {
                // odd extension node
                let shared_nibbles = &nibbles[2..];
                let extension_length = 1 + shared_nibbles.len();
                if nibble == key[key_index]
                    && Self::concat_nibbles(shared_nibbles.to_vec())
                        == &key[key_index + 1..key_index + extension_length]
                {
                    let new_expected_root = dec.at(1).unwrap().as_val::<Vec<u8>>().unwrap();
                    return Self::_verify_trie_proof(
                        new_expected_root.into(),
                        key,
                        proof,
                        key_index + extension_length,
                        proof_index + 1,
                        expected_value,
                    );
                }
            } else {
                panic!("This should not be reached if the proof has the correct format");
            }
        } else {
            panic!("This should not be reached if the proof has the correct format");
        }

        expected_value.len() == 0
    }
}
