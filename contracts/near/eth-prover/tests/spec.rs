mod utils;
use eth_types::*;
use near_sdk::borsh::BorshSerialize;
use near_sdk::testing_env;
use utils::{
    get_context, init_eth_client, init_eth_prover, read_block, AddBlockHeaderArgs,
    AssertEthbridgeHashArgs,
};
use workspaces::{Account, Contract};

async fn setup_factory() -> (Account, Contract, Contract) {
    let worker = workspaces::sandbox().await.unwrap();
    let client = init_eth_client(&worker, true).await;
    let prover = init_eth_prover(&worker, client.id().clone()).await;
    (worker.root_account().unwrap(), client, prover)
}

#[tokio::test]
#[cfg(feature = "eip1559")]
async fn block_hash_safe_from_eth_client() {
    let (alice, client, prover) = setup_factory().await;

    let safe_block = read_block("../eth-client/src/data/12965000.json".to_string());
    let unsafe_block1 = read_block("../eth-client/src/data/12965001.json".to_string());
    let unsafe_block2 = read_block("../eth-client/src/data/12965002.json".to_string());
    for i in 12965001..12965006 {
        let block = read_block(format!("../eth-client/src/data/{}.json", i));
        let add_block_header_args = AddBlockHeaderArgs {
            block_header: block.header(),
            dag_nodes: block.to_double_node_with_merkle_proof_vec(),
        };

        let _result = alice
            .call(client.id(), "add_block_header")
            .args(add_block_header_args.try_to_vec().unwrap())
            .max_gas()
            .transact()
            .await
            .unwrap();
    }

    testing_env!(get_context(vec![])); // For use rlp::decode, eth_types::near_keccak256
    let header: BlockHeader = rlp::decode(safe_block.header().as_slice()).unwrap();
    let assert_ethclient_hash_args = AssertEthbridgeHashArgs {
        block_number: header.number,
        expected_block_hash: header.hash.unwrap(),
    };

    let result = alice
        .call(prover.id(), "assert_ethclient_hash")
        .args(assert_ethclient_hash_args.try_to_vec().unwrap())
        .max_gas()
        .transact()
        .await
        .unwrap();

    assert_eq!(result.raw_bytes().unwrap(), b"\x01".to_vec()); // true

    let header: BlockHeader = rlp::decode(unsafe_block1.header().as_slice()).unwrap();
    let assert_ethclient_hash_args = AssertEthbridgeHashArgs {
        block_number: header.number,
        expected_block_hash: header.hash.unwrap(),
    };

    let result = alice
        .call(prover.id(), "assert_ethclient_hash")
        .args(assert_ethclient_hash_args.try_to_vec().unwrap())
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert_eq!(result.raw_bytes().unwrap(), b"\x00".to_vec()); // false since block is not 10 block away

    let header: BlockHeader = rlp::decode(unsafe_block2.header().as_slice()).unwrap();
    let assert_ethclient_hash_args = AssertEthbridgeHashArgs {
        block_number: header.number,
        expected_block_hash: header.hash.unwrap(),
    };

    let result = alice
        .call(prover.id(), "assert_ethclient_hash")
        .args(assert_ethclient_hash_args.try_to_vec().unwrap())
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert_eq!(result.raw_bytes().unwrap(), b"\x00".to_vec()); // false since block is not 10 block away

    let header: BlockHeader = rlp::decode(safe_block.header().as_slice()).unwrap();
    let assert_ethclient_hash_args = AssertEthbridgeHashArgs {
        block_number: header.number - 1,
        expected_block_hash: header.hash.unwrap(),
    };
    let result = alice
        .call(prover.id(), "assert_ethclient_hash")
        .args(assert_ethclient_hash_args.try_to_vec().unwrap())
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert_eq!(result.raw_bytes().unwrap(), b"\x00".to_vec()); // false since block number is incorrect

    let header: BlockHeader = rlp::decode(safe_block.header().as_slice()).unwrap();
    let block_number = header.number;
    let header: BlockHeader = rlp::decode(unsafe_block1.header().as_slice()).unwrap();
    let assert_ethclient_hash_args = AssertEthbridgeHashArgs {
        block_number: block_number,
        expected_block_hash: header.hash.unwrap(),
    };
    let result = alice
        .call(prover.id(), "assert_ethclient_hash")
        .args(assert_ethclient_hash_args.try_to_vec().unwrap())
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert_eq!(result.raw_bytes().unwrap(), b"\x00".to_vec()); // false since block hash is incorrect
}
