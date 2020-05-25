use crate::{BtcClientContract, BlockHeader, HashStr, MerkleProof};
use std::collections::HashMap;
use near_sdk::{env};

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // part of writing unit tests is setting up a mock context
    // in this example, this is only needed for env::log in the contract
    // this is also a useful list to peek at when wondering what's available in env::*
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice.testnet".to_string(),
            signer_account_id: "robert.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "jane.testnet".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn accept_header() {
        let context = get_context(vec![], false);
        testing_env!(context);

        let mut contract = BtcClientContract { 
            most_recent_block_hash: HashStr { value: "".to_string() }, 
            blocks: HashMap::new() 
        };
        // Block 0.
        let block_header = BlockHeader{ 
            block_hash: HashStr{ value: "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f".to_string() },
            version: 1,
            prev_block_hash: HashStr{ value:"0000000000000000000000000000000000000000000000000000000000000000".to_string() },
            merkle_root_hash: HashStr{ value:"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b".to_string() },
            time: 1231006505,
            n_bits: 486604799,
            nonce: 2083236893
        };
        contract.accept_header(block_header);
    }

    #[test]
    fn verify_tx() {
        let context = get_context(vec![], false);
        testing_env!(context);

        let mut contract = BtcClientContract { 
            most_recent_block_hash: HashStr { value: "".to_string() }, 
            blocks: HashMap::new() 
        };

        // Block 100000.
        let block_header_100000 = BlockHeader{ 
            block_hash: HashStr{ value: "000000000003ba27aa200b1cecaad478d2b00432346c3f1f3986da1afd33e506".to_string() },
            version: 1,
            prev_block_hash: HashStr{ value:"000000000002d01c1fccc21636b607dfd930d31d01c3a62104612a1719011250".to_string() },
            merkle_root_hash: HashStr{ value:"f3e94742aca4b5ef85488dc37c06c3282295ffec960994b2c0d5ac2a25a95766".to_string() },
            time: 1293623863,
            n_bits: 453281356,
            nonce: 274148111
        };

        // Block 100001.
        let block_header_100001 = BlockHeader{ 
            block_hash: HashStr{ value: "00000000000080b66c911bd5ba14a74260057311eaeb1982802f7010f1a9f090".to_string() },
            version: 1,
            prev_block_hash: HashStr{ value:"000000000003ba27aa200b1cecaad478d2b00432346c3f1f3986da1afd33e506".to_string() },
            merkle_root_hash: HashStr{ value:"7fe79307aeb300d910d9c4bec5bacb4c7e114c7dfd6789e19f3a733debb3bb6a".to_string() },
            time: 1293624404,
            n_bits: 453281356,
            nonce: 2613872960
        };

        // Block 100002.
        let block_header_100002 = BlockHeader{ 
            block_hash: HashStr{ value: "0000000000013b8ab2cd513b0261a14096412195a72a0c4827d229dcc7e0f7af".to_string() },
            version: 1,
            prev_block_hash: HashStr{ value:"00000000000080b66c911bd5ba14a74260057311eaeb1982802f7010f1a9f090".to_string() },
            merkle_root_hash: HashStr{ value:"2fda58e5959b0ee53c5253da9b9f3c0c739422ae04946966991cf55895287552".to_string() },
            time: 1293625051,
            n_bits: 453281356,
            nonce: 2478813466
        };
        
        // Block 100003.
        let block_header_100003 = BlockHeader{ 
            block_hash: HashStr{ value: "000000000002a0a74129007b1481d498d0ff29725e9f403837d517683abac5e1".to_string() },
            version: 1,
            prev_block_hash: HashStr{ value:"0000000000013b8ab2cd513b0261a14096412195a72a0c4827d229dcc7e0f7af".to_string() },
            merkle_root_hash: HashStr{ value:"5f17bef3616d703f27d05f0125b1720ba925a61fc44008dac6c438d609fc7937".to_string() },
            time: 1293625258,
            n_bits: 453281356,
            nonce: 2194287892
        };

        // Block 100004.
        let block_header_100004 = BlockHeader{ 
            block_hash: HashStr{ value: "000000000000b0b8b4e8105d62300d63c8ec1a1df0af1c2cdbd943b156a8cd79".to_string() },
            version: 1,
            prev_block_hash: HashStr{ value:"000000000002a0a74129007b1481d498d0ff29725e9f403837d517683abac5e1".to_string() },
            merkle_root_hash: HashStr{ value:"a5c67f0b44c1a342dec9135a566fd93f0c728452d1dbf850f4f9a47af82aed77".to_string() },
            time: 1293625358,
            n_bits: 453281356,
            nonce: 2731388424
        };

        // Block 100005.
        let block_header_100005 = BlockHeader{ 
            block_hash: HashStr{ value: "000000000000dab0130bbcc991d3d7ae6b81aa6f50a798888dfe62337458dc45".to_string() },
            version: 1,
            prev_block_hash: HashStr{ value:"000000000000b0b8b4e8105d62300d63c8ec1a1df0af1c2cdbd943b156a8cd79".to_string() },
            merkle_root_hash: HashStr{ value:"63194f18be0af63f2c6bc9dc0f777cbefed3d9415c4af83f3ee3a3d669c00cb5".to_string() },
            time: 1293625703,
            n_bits: 453281356,
            nonce: 1667081359
        };
        
        let block_header_100006 = BlockHeader{ 
            block_hash: HashStr{ value: "0000000000009b958a82c10804bd667722799cc3b457bc061cd4b7779110cd60".to_string() },
            version: 1,
            prev_block_hash: HashStr{ value:"000000000000dab0130bbcc991d3d7ae6b81aa6f50a798888dfe62337458dc45".to_string() },
            merkle_root_hash: HashStr{ value:"75c9a01add867b2fd57b2b70b246a9d8eaf14727673ccb01c85c974f2f8a7221".to_string() },
            time: 1293626048,
            n_bits: 453281356,
            nonce: 1834239867
        };

        contract.accept_header(block_header_100000);
        contract.accept_header(block_header_100001);
        contract.accept_header(block_header_100002);
        contract.accept_header(block_header_100003);
        contract.accept_header(block_header_100004);
        contract.accept_header(block_header_100005);
        contract.accept_header(block_header_100006);

        let merkle_proof = MerkleProof { 
            tx_hash: HashStr{ value: "fff2525b8931402dd09222c50775608f75787bd2b87e56995a7bdd30f79702c4".to_string() },
            tx_index: 1,
            siblings: vec![
                HashStr{ value: "8c14f0db3df150123e6f3dbbf30f8b955a8249b62ac1d1ff16284aefa3d06d87".to_string() },
                HashStr{ value: "8e30899078ca1813be036a073bbf80b86cdddde1c96e9e9c99e9e3782df4ae49".to_string() }
            ]
        };

        let success = contract.verify_tx(&merkle_proof, &"000000000003ba27aa200b1cecaad478d2b00432346c3f1f3986da1afd33e506".to_string());
        assert_eq!(true, success);
    }

    /**
        Transactions for block 100K:
        txs: [
            '8c14f0db3df150123e6f3dbbf30f8b955a8249b62ac1d1ff16284aefa3d06d87',
            'fff2525b8931402dd09222c50775608f75787bd2b87e56995a7bdd30f79702c4',
            '6359f0868171b1d194cbee1af2f16ea598ae8fad666d9b012c8ed2b79a236ec4',
            'e9a66845e05d5abc0ad04ec80f774a7e585c6e8db975962d069a522137b80c1d'
        ],
        merkleRoot: 'f3e94742aca4b5ef85488dc37c06c3282295ffec960994b2c0d5ac2a25a95766',
        hashLeftPair: 'ccdafb73d8dcd0173d5d5c3c9a0770d0b3953db889dab99ef05b1907518cb815',
        hashRightPair: '8e30899078ca1813be036a073bbf80b86cdddde1c96e9e9c99e9e3782df4ae49'
     */
    #[test]
    fn calculate_merkle_root() {
        let context = get_context(vec![], false);
        testing_env!(context);

        let merkle_proof = MerkleProof { 
            tx_hash: HashStr{ value: "fff2525b8931402dd09222c50775608f75787bd2b87e56995a7bdd30f79702c4".to_string() },
            tx_index: 1,
            siblings: vec![
                HashStr{ value: "8c14f0db3df150123e6f3dbbf30f8b955a8249b62ac1d1ff16284aefa3d06d87".to_string() },
                HashStr{ value: "8e30899078ca1813be036a073bbf80b86cdddde1c96e9e9c99e9e3782df4ae49".to_string() }
            ]
        };

        let merkle_root_hash = BtcClientContract::calculate_merkle_root(&merkle_proof);
        assert_eq!("f3e94742aca4b5ef85488dc37c06c3282295ffec960994b2c0d5ac2a25a95766", merkle_root_hash.value);
    }

    #[test]
    fn calculate_block_hash() {
        let context = get_context(vec![], false);
        testing_env!(context);
        
        // Block 0.
        let block_header = BlockHeader{ 
            block_hash: HashStr{ value: "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f".to_string() },
            version: 1,
            prev_block_hash: HashStr{ value:"0000000000000000000000000000000000000000000000000000000000000000".to_string() },
            merkle_root_hash: HashStr{ value:"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b".to_string() },
            time: 1231006505,
            n_bits: 486604799,
            nonce: 2083236893
        };
        let block_hash = BtcClientContract::calculate_block_hash(&block_header);
        assert_eq!("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f", block_hash);
    }
}
