#[cfg(test)]
mod tests1 {
    use crate::EthProver;
    use hex::{FromHex, ToHex};
    use near_sdk::PromiseOrValue;
    use rlp::RlpStream;
    use serde::{Deserialize, Deserializer};

    #[derive(Debug)]
    struct Hex(pub Vec<u8>);

    impl<'de> Deserialize<'de> for Hex {
        fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
        where
            D: Deserializer<'de>,
        {
            let mut s = <String as Deserialize>::deserialize(deserializer)?;
            if s.starts_with("0x") {
                s = s[2..].to_string();
            }
            if s.len() % 2 == 1 {
                s.insert_str(0, "0");
            }
            Ok(Hex(Vec::from_hex(&s).map_err(|err| {
                serde::de::Error::custom(err.to_string())
            })?))
        }
    }

    // TESTS

    use near_sdk::{testing_env, VMContext};

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
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: near_sdk::Gas(10u64.pow(18)),
            random_seed: vec![1; 32].try_into().unwrap(),
            view_config: None,
            output_data_receivers: vec![],
        }
    }
   
        #[test]
        pub fn test_state_proof() {
            testing_env!(get_context(vec![]));

            //let expected_value = "01";
            let expected_root = "9dc8b927bc1f203931c70cc3850246046859c40e0044964753b28ff41285b75d"; //state root
            let key = "ec92a9aa5b0091a625b3467da991181b7f6a3871356857ee2e3d726fcf075c83"; // contract add
            let proof_rlp = vec!["63168682a314606854cba72eda7c136c685366c251187aa59a1ab5aaa802ecd4",
                                 "4f063070027f6f9734cfe049ac1875b1ad9ae7f9e0ca98961afe97658d4357c4",
                                 "25eab5759163d28627d2c73694878fc56ea434664d2416bec721a63b4348eb6a"
                                ];

            let expected_root = hex::decode(expected_root).unwrap().into();
            let key = hex::decode(key).unwrap();
            let proof = proof_rlp
                .into_iter()
                .map(|x| hex::decode(x).unwrap())
                .collect();
            //let expected_value = hex::decode(expected_value).unwrap();
            let result = EthProver::verify_trie_proof(expected_root, key, proof);

            // assert_eq!(
            //     EthProver::verify_trie_proof(expected_root, key, proof),
            //     expected_value
            // );
            println!("{:?}", result);
        }
    
}