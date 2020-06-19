mod utils;
use utils::{new_root, RuntimeStandalone, ExternalUser, read_block, AddBlockHeaderArgs, AssertEthbridgeHashArgs, ntoy, get_context};
use borsh::{BorshDeserialize, BorshSerialize};
use eth_types::*;
use near_sdk::{testing_env, VMContext, MockedBlockchain};

fn setup_factory() -> (RuntimeStandalone, ExternalUser) {
    let (mut r, near) = new_root("near".into());
    near.init_eth_client(&mut r, "eth-client".to_string(), true).unwrap();
    near.init_eth_prover(&mut r, "eth-prover".to_string(), "eth-client".to_string()).unwrap();
    (r, near)
}

#[test]
fn block_hash_safe_from_eth_client() {
    let (mut r, near) = setup_factory();
    let block = read_block("../eth-client/src/data/8996777.json".to_string());
    let add_block_header_args = AddBlockHeaderArgs {
        block_header: block.header(),
        dag_nodes: block.to_double_node_with_merkle_proof_vec()
    };
    near.function_call(&mut r, "eth-client", "add_block_header", &add_block_header_args.try_to_vec().unwrap(), ntoy(0)).unwrap();

    testing_env!(get_context(vec![], false)); // For use rlp::decode, eth_types::near_keccak256
    let header: BlockHeader = rlp::decode(block.header().as_slice()).unwrap();
    let assert_ethbridge_hash_args = AssertEthbridgeHashArgs {
        block_number: header.number,
        expected_block_hash: header.hash.unwrap(),
    };
    println!("{:?}", near.function_call(&mut r, "eth-prover", "assert_ethbridge_hash", &assert_ethbridge_hash_args.try_to_vec().unwrap(),ntoy(0)).unwrap());
}

