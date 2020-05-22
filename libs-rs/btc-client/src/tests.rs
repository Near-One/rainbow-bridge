use crate::{BtcClientContract, BlockHeader, HashStr};
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
        // TODO test other verifications.
    }

    #[test]
    fn calculate_block_hash() {
        let context = get_context(vec![], false);
        testing_env!(context);
        
        let block_header = BlockHeader{ 
            block_hash: HashStr{ value: "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f".to_string() },
            version: 1,
            prev_block_hash: HashStr{ value:"0000000000000000000000000000000000000000000000000000000000000000".to_string() },
            merkle_root_hash: HashStr{ value:"4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b".to_string() },
            time: 1231006505,
            n_bits: 486604799,
            nonce: 2083236893
        };
        let block_hash = BtcClientContract::calculate_block_hash(block_header);
        assert_eq!("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f", block_hash);
    }
}
