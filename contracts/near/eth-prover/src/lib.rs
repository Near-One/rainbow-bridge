use admin_controlled::Mask;
use borsh::{BorshDeserialize, BorshSerialize};
use eth_types::*;
use near_sdk::{env, ext_contract, near_bindgen, Gas, PanicOnDefault, PromiseOrValue};
use rlp::Rlp;

near_sdk::setup_alloc!();

type AccountId = String;

/// Gas to call block_hash_safe
const BLOCK_HASH_SAFE_GAS: Gas = 10_000_000_000_000;

/// Gas to call on_block_hash
const ON_BLOCK_HASH_GAS: Gas = 5_000_000_000_000;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct EthProver {
    bridge_smart_contract: AccountId,
    paused: Mask,
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

/// Get element at position `pos` from rlp encoded data,
/// and decode it as vector of bytes
fn get_vec(data: &Rlp, pos: usize) -> Vec<u8> {
    data.at(pos).unwrap().as_val::<Vec<u8>>().unwrap()
}

const PAUSE_VERIFY: Mask = 1;

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
            paused: Mask::default(),
        }
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
            BLOCK_HASH_SAFE_GAS,
        )
        .then(remote_self::on_block_hash(
            expected_block_hash,
            &env::current_account_id(),
            0,
            ON_BLOCK_HASH_GAS,
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
        self.check_not_paused(PAUSE_VERIFY);
        let log_entry: LogEntry = rlp::decode(log_entry_data.as_slice()).unwrap();
        let receipt: Receipt = rlp::decode(receipt_data.as_slice()).unwrap();
        let header: BlockHeader = rlp::decode(header_data.as_slice()).unwrap();

        // Verify log_entry included in receipt
        assert_eq!(receipt.logs[log_index as usize], log_entry);

        // Verify receipt included into header
        let data =
            Self::verify_trie_proof(header.receipts_root, rlp::encode(&receipt_index), proof);
        let verification_result = receipt_data == data;
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
            BLOCK_HASH_SAFE_GAS,
        )
        .then(remote_self::on_block_hash(
            header.hash.unwrap(),
            &env::current_account_id(),
            0,
            ON_BLOCK_HASH_GAS,
        ))
        .into()
    }

    /// Verify the proof recursively traversing through the key.
    /// Return the value at the end of the key, in case the proof is valid.
    ///
    /// @param expected_root is the expected root of the current node.
    /// @param key is the key for which we are proving the value.
    /// @param proof contains relevant information to verify data is valid
    ///
    /// Patricia Trie: https://eth.wiki/en/fundamentals/patricia-tree
    /// Patricia Img:  https://ethereum.stackexchange.com/questions/268/ethereum-block-architecture/6413#6413
    ///
    /// Verification:  https://github.com/slockit/in3/wiki/Ethereum-Verification-and-MerkleProof#receipt-proof
    /// Article:       https://medium.com/@ouvrard.pierre.alain/merkle-proof-verification-for-ethereum-patricia-tree-48f29658eec
    /// Python impl:   https://gist.github.com/mfornet/0ff283274c0162f1cca45966bccf69ee
    ///
    fn verify_trie_proof(expected_root: H256, key: Vec<u8>, proof: Vec<Vec<u8>>) -> Vec<u8> {
        let mut actual_key = vec![];
        for el in key {
            actual_key.push(el / 16);
            actual_key.push(el % 16);
        }
        Self::_verify_trie_proof((expected_root.0).0.into(), &actual_key, &proof, 0, 0)
    }

    fn _verify_trie_proof(
        expected_root: Vec<u8>,
        key: &Vec<u8>,
        proof: &Vec<Vec<u8>>,
        key_index: usize,
        proof_index: usize,
    ) -> Vec<u8> {
        let node = &proof[proof_index];

        if key_index == 0 {
            // trie root is always a hash
            assert_eq!(near_keccak256(node), expected_root.as_slice());
        } else if node.len() < 32 {
            // if rlp < 32 bytes, then it is not hashed
            assert_eq!(node.as_slice(), expected_root);
        } else {
            assert_eq!(near_keccak256(node), expected_root.as_slice());
        }

        let node = Rlp::new(&node.as_slice());

        if node.iter().count() == 17 {
            // Branch node
            if key_index == key.len() {
                assert_eq!(proof_index + 1, proof.len());
                get_vec(&node, 16)
            } else {
                let new_expected_root = get_vec(&node, key[key_index] as usize);
                Self::_verify_trie_proof(
                    new_expected_root,
                    key,
                    proof,
                    key_index + 1,
                    proof_index + 1,
                )
            }
        } else {
            // Leaf or extension node
            assert_eq!(node.iter().count(), 2);
            let path_u8 = get_vec(&node, 0);
            // Extract first nibble
            let head = path_u8[0] / 16;
            // assert!(0 <= head); is implicit because of type limits
            assert!(head <= 3);

            // Extract path
            let mut path = vec![];
            if head % 2 == 1 {
                path.push(path_u8[0] % 16);
            }
            for val in path_u8.into_iter().skip(1) {
                path.push(val / 16);
                path.push(val % 16);
            }
            assert_eq!(path.as_slice(), &key[key_index..key_index + path.len()]);

            if head >= 2 {
                // Extension node
                assert_eq!(proof_index + 1, proof.len());
                assert_eq!(key_index + path.len(), key.len());
                get_vec(&node, 1)
            } else {
                // Leaf node
                let new_expected_root = get_vec(&node, 1);
                Self::_verify_trie_proof(
                    new_expected_root,
                    key,
                    proof,
                    key_index + path.len(),
                    proof_index + 1,
                )
            }
        }
    }
}

admin_controlled::impl_admin_controlled!(EthProver, paused);

#[cfg(test)]
mod tests {
    use crate::EthProver;
    use near_sdk::{testing_env, MockedBlockchain};
    use near_sdk::{VMConfig, VMContext};

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice.near".to_string(),
            signer_account_id: "bob.near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol.near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            epoch_height: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
        }
    }

    #[test]
    pub fn test_verify_proof() {
        let vm_config = VMConfig::free();
        testing_env!(get_context(vec![], false), vm_config, Default::default());

        let expected_value = "f902a60183af4adfb9010000000000000000000000000000000000000000000000000000000000000000000800010000000000000002000100000000000000000000000000000000000000000000000000000008000008000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000010000000040000000000000000000000000000000200000000010000000000000000000000000000000000200080000000202000000000000000000000000000004000000000000002000000000000000000000000000000000000080000000000000000000000000000000000000000000200000004000000000000000000000000000000f9019bf89b94a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa0000000000000000000000000c22df065a81f6e0107e214991b9d7fb179d401b3a000000000000000000000000023ddd3e3692d1861ed57ede224608875809e127fa00000000000000000000000000000000000000000000000000000000005f5e100f8fc9423ddd3e3692d1861ed57ede224608875809e127ff863a0dd85dc56b5b4da387bf69c28ec19b1d66e793e0d51b567882fa31dc50bbd32c5a0000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48a0000000000000000000000000c22df065a81f6e0107e214991b9d7fb179d401b3b8800000000000000000000000000000000000000000000000000000000005f5e1000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000a6d616b6b652e6e65617200000000000000000000000000000000000000000000";
        let expected_root = "73733f420161b4189ea48140489bfada55d485fd580ab7e6f0f4b6de229f5177";
        let key = "820137";
        let proof_rlp = vec!["f90131a07dc6deefc13acb3456defc30824e9ba7d344e0fed67d6fe918a845ac6c7ff68ca00743abb9e8a2419e731aabac46e87dd9166ef04f4c0e17b607f699169fd16effa0de6439dd92daf3fe5ae984f9176a1d53f0ab2c03a73ea9ee2c94c79a87b82386a08eccde52f7cdcfa2207e2e8256bbd05a78fbab4f1b564a98a7f5b259b9fcb05da0196a72fd5279acc9146896618d5a134398bfc5d84063bcb2dc4f206bceb1526fa0daa3100c65bc47d986741898d7dfa1cc2f944d9f621b33a53d52047d98ab6e84a0126a9c69a2fb01312dffd739ee2a86c15106497d5e53314875e3a83c915b40c3a0b89b77f6776de33f0d291891d4271546d3b4946325f6fa66d38a1618f699b7b0a06b4f2fac50925da7c11ddac2321257cf157d426bffedcce8c3e999f8dd3902ff8080808080808080", "f871a073170337a44a638efb6d735150b3a06346b54b6176c9424307e6c1f4a4604131a0409f60141274adbaf1fd8808c432c599025a80763a61aca8710ba5416436c885a064d0127fe80ad8301e425eece21dd4811515312fad7e95b9ad4f853d003582a88080808080808080808080808080", "e4820001a022d9ed1b1940164d904d587080c9ca1d5ebb7e711211233bee7ecf6f0fba3d8e", "f8d1a0af41ab83382da16fba21a258c18a231957c14eb91ede9b75b089d37474efe1b8a0102af48a2d48aa200cb90bafdb43c3845ed09e2d34f333944ac7c172f2becec4a0644f776baaf4dd2a45e817c3b70ed881419f31d966debf0e2dac62426b1308eea08498703814dfa09c76b9f8dde1d5e3865b92b805e9ffd77d12cd8221497fe604a02a6a5cc557e67488aef767895f2ad789fa339aab229d7e94b78d6a8187989d3ea002ec005b9dafdfd58601cd6dc96fd958a8681981442c485c87142ae85acc1fd28080808080808080808080", "f90211a02e04ddc4ab28665d70404d04601838d03b219207d68a477e086144d5452b035ea07458b1e7734dcde7a48e763f57b533d39a9893b2fd05ece758eb95c45230b69aa05ce53207da7cb7efdfa60bfc57dc23a5469d2823ac6d94377fda37d7d6e77a23a0ba360fa8bb757bec0086b0d2973bb39de6b874ae3558e0b91eb54579022bc68fa023068af8cff2927c6c437840b4bee730c5ef2d918c0bb086b453da9071f3e3a8a05a6cb3455636113070e724682c1e852b564bc26195690b57adf95b03e453fb56a090e6afaa341c8c8583ba621e0f4369e0a36e488d167061bf0627c8de8e4b1b53a0c974bc6676b17c2e0e0016b86e2261cc69fa10b06ad22938f851674c853face6a0134724224ee173faf5807c3b963e4aff5d8435c2296230ba6fbdf222262ee7a5a0ea6f74b84a4ee7f7d557bb27e61c75ae30ead9092faab3b441c4e5055c8768d7a09e5ab3942cfe8410180611c7eddb2364ba2022e53971b50250636f2576f1528fa0fe27620c114d5ce5b8c96859b754cf0a40a9a0cb7a6c88a492ca906b09db6c5aa023558ac1d7facb4cb81d5ced13c9126b70a898a81b63fe52117792ed5bcadb06a00b52d0a4cd96595521a0783e6f17de8b9bdd15f1e50ab8b51179a98ae6df18e5a0462c433d431953bcead5900c4b372c60b20280d05366fe48de6384152cfc8da9a0a5a9b086a22dc344a40496a9de4ebab616fa87dfc77760eef7939ba51dd193cd80", "f902ad20b902a9f902a60183af4adfb9010000000000000000000000000000000000000000000000000000000000000000000800010000000000000002000100000000000000000000000000000000000000000000000000000008000008000000000000000000000200000000000000000000000000000000000000000000000000000000000000010000000010000000040000000000000000000000000000000200000000010000000000000000000000000000000000200080000000202000000000000000000000000000004000000000000002000000000000000000000000000000000000080000000000000000000000000000000000000000000200000004000000000000000000000000000000f9019bf89b94a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa0000000000000000000000000c22df065a81f6e0107e214991b9d7fb179d401b3a000000000000000000000000023ddd3e3692d1861ed57ede224608875809e127fa00000000000000000000000000000000000000000000000000000000005f5e100f8fc9423ddd3e3692d1861ed57ede224608875809e127ff863a0dd85dc56b5b4da387bf69c28ec19b1d66e793e0d51b567882fa31dc50bbd32c5a0000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48a0000000000000000000000000c22df065a81f6e0107e214991b9d7fb179d401b3b8800000000000000000000000000000000000000000000000000000000005f5e1000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000a6d616b6b652e6e65617200000000000000000000000000000000000000000000"];

        let expected_root = hex::decode(expected_root).unwrap().into();
        let key = hex::decode(key).unwrap();
        let proof = proof_rlp
            .into_iter()
            .map(|x| hex::decode(x).unwrap())
            .collect();
        let expected_value = hex::decode(expected_value).unwrap();

        assert_eq!(
            EthProver::verify_trie_proof(expected_root, key, proof),
            expected_value
        );
    }
}
