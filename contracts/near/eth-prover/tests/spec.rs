mod utils;
use borsh::BorshSerialize;
use eth_types::*;
use near_primitives::transaction::ExecutionStatus;
use near_sdk::{testing_env, MockedBlockchain};
use utils::{
    get_context, new_root, ntoy, read_block, AddBlockHeaderArgs, AssertEthbridgeHashArgs,
    ExternalUser, RuntimeStandalone,
};

fn setup_factory() -> (RuntimeStandalone, ExternalUser) {
    let (mut r, near) = new_root("near".into());
    near.init_eth_client(&mut r, "eth-client".to_string(), true)
        .unwrap();
    near.init_eth_prover(&mut r, "eth-prover".to_string(), "eth-client".to_string())
        .unwrap();
    (r, near)
}

#[test]
fn block_hash_safe_from_eth_client() {
    let (mut r, near) = setup_factory();

    let safe_block = read_block("../eth-client/src/data/10234001.json".to_string());
    let unsafe_block1 = read_block("../eth-client/src/data/10234002.json".to_string());
    let unsafe_block2 = read_block("../eth-client/src/data/10234003.json".to_string());
    for i in 10234002..10234012 {
        let block = read_block(format!("../eth-client/src/data/{}.json", i));
        let add_block_header_args = AddBlockHeaderArgs {
            block_header: block.header(),
            dag_nodes: block.to_double_node_with_merkle_proof_vec(),
        };
        near.function_call(
            &mut r,
            "eth-client",
            "add_block_header",
            &add_block_header_args.try_to_vec().unwrap(),
            ntoy(0),
        )
        .unwrap();
    }

    testing_env!(get_context(vec![], false)); // For use rlp::decode, eth_types::near_keccak256
    let header: BlockHeader = rlp::decode(safe_block.header().as_slice()).unwrap();
    let assert_ethclient_hash_args = AssertEthbridgeHashArgs {
        block_number: header.number,
        expected_block_hash: header.hash.unwrap(),
    };
    let res = near
        .function_call(
            &mut r,
            "eth-prover",
            "assert_ethclient_hash",
            &assert_ethclient_hash_args.try_to_vec().unwrap(),
            ntoy(0),
        )
        .unwrap();
    assert_eq!(res.status, ExecutionStatus::SuccessValue(b"\x01".to_vec())); // true

    let header: BlockHeader = rlp::decode(unsafe_block1.header().as_slice()).unwrap();
    let assert_ethclient_hash_args = AssertEthbridgeHashArgs {
        block_number: header.number,
        expected_block_hash: header.hash.unwrap(),
    };
    let res = near
        .function_call(
            &mut r,
            "eth-prover",
            "assert_ethclient_hash",
            &assert_ethclient_hash_args.try_to_vec().unwrap(),
            ntoy(0),
        )
        .unwrap();
    assert_eq!(res.status, ExecutionStatus::SuccessValue(b"\x00".to_vec())); // false since block is not 10 block away

    let header: BlockHeader = rlp::decode(unsafe_block2.header().as_slice()).unwrap();
    let assert_ethclient_hash_args = AssertEthbridgeHashArgs {
        block_number: header.number,
        expected_block_hash: header.hash.unwrap(),
    };
    let res = near
        .function_call(
            &mut r,
            "eth-prover",
            "assert_ethclient_hash",
            &assert_ethclient_hash_args.try_to_vec().unwrap(),
            ntoy(0),
        )
        .unwrap();
    assert_eq!(res.status, ExecutionStatus::SuccessValue(b"\x00".to_vec())); // false since block is not 10 block away

    let header: BlockHeader = rlp::decode(safe_block.header().as_slice()).unwrap();
    let assert_ethclient_hash_args = AssertEthbridgeHashArgs {
        block_number: header.number - 1,
        expected_block_hash: header.hash.unwrap(),
    };
    let res = near
        .function_call(
            &mut r,
            "eth-prover",
            "assert_ethclient_hash",
            &assert_ethclient_hash_args.try_to_vec().unwrap(),
            ntoy(0),
        )
        .unwrap();
    assert_eq!(res.status, ExecutionStatus::SuccessValue(b"\x00".to_vec())); // false since block number is incorrect

    let header: BlockHeader = rlp::decode(safe_block.header().as_slice()).unwrap();
    let block_number = header.number;
    let header: BlockHeader = rlp::decode(unsafe_block1.header().as_slice()).unwrap();
    let assert_ethclient_hash_args = AssertEthbridgeHashArgs {
        block_number: block_number,
        expected_block_hash: header.hash.unwrap(),
    };
    let res = near
        .function_call(
            &mut r,
            "eth-prover",
            "assert_ethclient_hash",
            &assert_ethclient_hash_args.try_to_vec().unwrap(),
            ntoy(0),
        )
        .unwrap();
    assert_eq!(res.status, ExecutionStatus::SuccessValue(b"\x00".to_vec())); // false since block hash is incorrect
}
