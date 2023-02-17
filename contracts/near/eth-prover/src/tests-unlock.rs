#[cfg(test)]
mod tests_unlock_proof {
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
            let key = "1c8ba9af7041ec3098c4d818db9972f67827520c1db7d022f6c3041b6f40ecc3"; // contract add
            let proof_rlp = vec!["f90211a0786e2b7a3a24cfafe31c1cac9b0cc4b57cfb9e27264b9aeb8ba52d48181fd013a0bbf8c0df9d3837792180786e217cbcbfb6cb6c1ee790ec0dba83fc05248f2627a060d5477cb7243063a66f6533e342632aea890e13cf45c0144ad7f8254bb74242a02dd5604620fcf3990474b172d1bb5653bf7bfc9ee1d269e765da3f7a91ac2ec3a04134b666e4a8e3e28701e33bed36680ae025cc5f89f8d29457975148338017c4a07bf7ed874ff02f5938267592e1a3cfca0ca8d7d4b6fccd617b815fad456c2c53a076515bac0f65924da357aba2f6c312472a9ef94d1c459f2cd0380b84a91d46aca0f49dd3a32c4adb7b6f02188b32a7823c79bb5b2ed1f3cdf4da11c36304ae4e25a05ba06e342a112365ced395b4bf830d148ad1e6a787912a9a603293a4e079e3f5a029f967b2d7ed040cda92ab4905f2976636a7bb16ba850679eca942e02d5b3458a024bdf35e1d8fd46cdee76420d508c722d7f47825356620c266094f6c66c67f7ea070a8abf4345f7148bbcbd0f443fe00cfed0a17ab828b7051cb3ede4bee4499b1a00d9cd9aa39cdd4a91bcd4feb8c73db017dce78ec82ba6f14287775567615f247a02f3d28876d69fd8945b3fdeff87186bcb97c236ccefa3b6182507bc6e504606fa03b4078e843b8927c29753dfc91a44c246cb22ea3673a3bad8dab7c8f6b8ce226a061983eaffadbdc7d6e7a0f9d9f5c1c64f983a4a6724aaa2126a0b6c03c984f3280",
                                 "f901d1a0269a0b881e99b091fd1f85a9e44ee4daf1bfaa127412213b0ec1edacf2fcb4e0a0e7f129286b917093dd713ebf19c11b699dd3b7e2c8dd81b96da000d8929e6bd7a042e1e974e7e7b3829b6fd86c08ec48c92789a4c5b29aaba295ac47eb78da8979a0af69958bb871e61c3dab996738ae596fd6d8b0c6dd126361053ca23c67576aa6a0957f511c654ba45de24daa58d5069b8e854e1af4230f5e00ff80942f32179f9280a002cca7c62c8aaf39f3300a3694c5df5909d2365d68935dead512cec79b859546a0c150cdf6bf843bf02c321ad13d4b680a5ac6816a93b607b68b3d2944645cc573a0d44663526783865e11155d5490966374eaf24421c59522f6350769715d83f5f280a028d3420f6f3cfd2eff10d769032ec92a9ce6a26004a4b5e432577f7d218cb8eea095fcf2b12e8b70bb0c075d41576e00242769a18622ab1d936326fa053aeaa501a0ecf0ef75da3fd6cd5bc5a73049bbbdedd2fb0ccdb86bb07895d012b1b93354d5a02c8f51d330045fd36258abd5dc39dbc9ac14103ca91925bed4f55066f7d51830a024fff2a5c88286da53434ffeafc0962a9c2230a26c303850a04853096882be92a0a0eb992284319d09507645086aaf48f97428294e89aab782574993ed0543caef80",
                                 "f851808080808080a05a173679dbc21d5c5b16c40e4d0fdd5ec92f602c2db2adbe71c9d4b2aedb585480a04cd868f531723c2438dce2df71b16e2d4d6f49299867484a662073b40aaac5ca8080808080808080",
                                 "e19f3ba9af7041ec3098c4d818db9972f67827520c1db7d022f6c3041b6f40ecc301"
                                ];

            let expected_root = hex::decode(expected_root).unwrap().into();
            let key = hex::decode(key).unwrap().into();
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