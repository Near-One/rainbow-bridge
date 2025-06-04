#[cfg(test)]
mod tests_storage_proof {
    use crate::EthProver;
    use eth_types::H256;
    use near_sdk::serde_json;
    use near_sdk::PromiseOrValue;
    use rlp::Rlp;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    pub struct JsonProof {
        #[serde(with = "hex::serde")]
        pub header_data: Vec<u8>,
        pub account_proof: Vec<String>, // account proof
        #[serde(with = "hex::serde")]
        pub contract_address: Vec<u8>, // eth address
        #[serde(with = "hex::serde")]
        pub expected_account_state: Vec<u8>, // encoded account state
        #[serde(with = "hex::serde")]
        pub storage_key_hash: Vec<u8>, // keccak256 of storage key
        pub storage_proof: Vec<String>, // storage proof
        #[serde(with = "hex::serde")]
        pub expected_storage_value: Vec<u8>, // storage value
        pub min_header_height: String,
        pub max_header_height: String,
        pub skip_bridge_call: bool,
    }

    #[derive(Debug, Deserialize)]
    pub struct StorageProof {
        pub header_data: Vec<u8>,
        pub account_proof: Vec<Vec<u8>>,     // account proof
        pub contract_address: Vec<u8>,       // eth address
        pub expected_account_state: Vec<u8>, // encoded account state
        pub storage_key_hash: Vec<u8>,       // keccak256 of storage key
        pub storage_proof: Vec<Vec<u8>>,     // storage proof
        pub expected_storage_value: Vec<u8>, // storage value
        pub min_header_height: Option<u64>,
        pub max_header_height: Option<u64>,
        pub skip_bridge_call: bool,
    }

    pub fn get_json_proof(filename: String) -> JsonProof {
        let contents = std::fs::read_to_string(&filename).expect("Unable to read file");
        serde_json::from_str(&contents).expect("Unable to deserialize")
    }

    pub fn get_storage_proof(file_path: String) -> StorageProof {
        let json_proof: JsonProof = get_json_proof(file_path);

        let header_data = json_proof.header_data;
        let contract_address = json_proof.contract_address;
        let account_proof = json_proof
            .account_proof
            .into_iter()
            .map(|x| hex::decode(x).unwrap())
            .collect();
        let expected_account_state = json_proof.expected_account_state;
        let storage_key_hash = json_proof.storage_key_hash;
        let storage_proof = json_proof
            .storage_proof
            .into_iter()
            .map(|x| hex::decode(x).unwrap())
            .collect();

        StorageProof {
            header_data,
            account_proof,
            contract_address,
            expected_account_state,
            storage_key_hash,
            storage_proof,
            expected_storage_value: json_proof.expected_storage_value,
            min_header_height: None,
            max_header_height: None,
            skip_bridge_call: json_proof.skip_bridge_call,
        }
    }

    // TESTS

    use near_sdk::{testing_env, NearToken, VMContext};

    fn get_context(input: Vec<u8>) -> VMContext {
        VMContext {
            current_account_id: "alice.near".parse().unwrap(),
            signer_account_id: "bob.near".parse().unwrap(),
            signer_account_pk: "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp"
                .parse()
                .unwrap(),
            predecessor_account_id: "carol.near".parse().unwrap(),
            input,
            block_index: 0,
            block_timestamp: 0,
            epoch_height: 0,
            account_balance: NearToken::from_near(0),
            account_locked_balance: NearToken::from_near(0),
            storage_usage: 0,
            attached_deposit: NearToken::from_near(0),
            prepaid_gas: near_sdk::Gas::from_tgas(1_000_000),
            random_seed: vec![1; 32].try_into().unwrap(),
            view_config: None,
            output_data_receivers: vec![],
        }
    }

    #[test]
    pub fn test_verify_trie_proof() {
        testing_env!(get_context(vec![]));
        let test_data = get_storage_proof(String::from("./src/test_data/storageProof.json"));
        let storage_hash: H256 = Rlp::new(&test_data.expected_account_state)
            .val_at(2)
            .unwrap();
        assert_eq!(
            EthProver::verify_trie_proof(
                storage_hash,
                test_data.storage_key_hash,
                test_data.storage_proof
            ),
            test_data.expected_storage_value
        );
    }

    #[test]
    pub fn test_verify_storage_proof() {
        testing_env!(get_context(vec![]));
        let contract = EthProver::init("ethbridge".to_string());
        let test_data = get_storage_proof(String::from("./src/test_data/storageProof.json"));
        if let PromiseOrValue::Value(true) = contract.verify_storage_proof(
            test_data.header_data,
            test_data.account_proof,
            test_data.contract_address,
            test_data.expected_account_state,
            test_data.storage_key_hash,
            test_data.storage_proof,
            test_data.expected_storage_value,
            test_data.min_header_height,
            test_data.max_header_height,
            test_data.skip_bridge_call,
        ) {
        } else {
            panic!();
        }
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    pub fn test_verify_storage_proof_with_wrong_account_proof() {
        testing_env!(get_context(vec![]));
        let contract = EthProver::init("ethbridge".to_string());
        let test_data = get_storage_proof(String::from(
            "./src/test_data/storageProofWithIncorrectAccountProof.json",
        ));
        if let PromiseOrValue::Value(true) = contract.verify_storage_proof(
            test_data.header_data,
            test_data.account_proof,
            test_data.contract_address,
            test_data.expected_account_state,
            test_data.storage_key_hash,
            test_data.storage_proof,
            test_data.expected_storage_value,
            test_data.min_header_height,
            test_data.max_header_height,
            test_data.skip_bridge_call,
        ) {
        } else {
            panic!();
        }
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    pub fn test_verify_storage_proof_with_wrong_state_proof() {
        testing_env!(get_context(vec![]));
        let contract = EthProver::init("ethbridge".to_string());
        let test_data = get_storage_proof(String::from(
            "./src/test_data/storageProofWithIncorrectStateProof.json",
        ));
        if let PromiseOrValue::Value(true) = contract.verify_storage_proof(
            test_data.header_data,
            test_data.account_proof,
            test_data.contract_address,
            test_data.expected_account_state,
            test_data.storage_key_hash,
            test_data.storage_proof,
            test_data.expected_storage_value,
            test_data.min_header_height,
            test_data.max_header_height,
            test_data.skip_bridge_call,
        ) {
        } else {
            panic!();
        }
    }

    #[test]
    #[should_panic(expected = "explicit panic")]
    pub fn test_verify_storage_proof_with_wrong_value() {
        testing_env!(get_context(vec![]));
        let contract = EthProver::init("ethbridge".to_string());
        let test_data = get_storage_proof(String::from(
            "./src/test_data/storageProofWithIncorrectValue.json",
        ));
        if let PromiseOrValue::Value(true) = contract.verify_storage_proof(
            test_data.header_data,
            test_data.account_proof,
            test_data.contract_address,
            test_data.expected_account_state,
            test_data.storage_key_hash,
            test_data.storage_proof,
            test_data.expected_storage_value,
            test_data.min_header_height,
            test_data.max_header_height,
            test_data.skip_bridge_call,
        ) {
        } else {
            panic!();
        }
    }

    #[test]
    pub fn test_verify_storage_proof_with_false_value() {
        testing_env!(get_context(vec![]));
        let contract = EthProver::init("ethbridge".to_string());
        let test_data = get_storage_proof(String::from(
            "./src/test_data/storageProofWithFalseValue.json",
        ));
        if let PromiseOrValue::Value(true) = contract.verify_storage_proof(
            test_data.header_data,
            test_data.account_proof,
            test_data.contract_address,
            test_data.expected_account_state,
            test_data.storage_key_hash,
            test_data.storage_proof,
            test_data.expected_storage_value,
            test_data.min_header_height,
            test_data.max_header_height,
            test_data.skip_bridge_call,
        ) {
        } else {
            panic!("storage_value != expected_storage_value");
        }
    }

    #[test]
    #[should_panic(expected = "explicit panic")]
    pub fn test_verify_storage_proof_with_wrong_account_data() {
        testing_env!(get_context(vec![]));
        let contract = EthProver::init("ethbridge".to_string());
        // Below Data for Block: 8487596 other than account data
        let test_data = get_storage_proof(String::from(
            "./src/test_data/storageProofWithIncorrectAccountState.json",
        ));
        if let PromiseOrValue::Value(true) = contract.verify_storage_proof(
            test_data.header_data,
            test_data.account_proof,
            test_data.contract_address,
            test_data.expected_account_state,
            test_data.storage_key_hash,
            test_data.storage_proof,
            test_data.expected_storage_value,
            test_data.min_header_height,
            test_data.max_header_height,
            test_data.skip_bridge_call,
        ) {
        } else {
            panic!();
        }
    }

    #[test]
    pub fn test_verify_storage_proof_with_future_header_data() {
        testing_env!(get_context(vec![]));
        let contract = EthProver::init("ethbridge".to_string());
        // Header-Data[state root-> block: 8487596 || other data in Header-data -> block: 8492954]
        let test_data = get_storage_proof(String::from(
            "./src/test_data/storageProofWithIncorrectHeaderExceptStateRoot.json",
        ));
        if let PromiseOrValue::Value(true) = contract.verify_storage_proof(
            test_data.header_data,
            test_data.account_proof,
            test_data.contract_address,
            test_data.expected_account_state,
            test_data.storage_key_hash,
            test_data.storage_proof,
            test_data.expected_storage_value,
            test_data.min_header_height,
            test_data.max_header_height,
            test_data.skip_bridge_call,
        ) {
        } else {
            panic!();
        }
    }
}
