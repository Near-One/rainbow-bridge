extern crate web3;

use super::*;
use near_bindgen::MockedBlockchain;
use near_bindgen::{VMContext, Config, testing_env};
use web3::futures::Future;
use web3::types::{BlockId, BlockNumber, Block};
use rlp::{RlpStream};

fn rlp_append<TX>(header: &Block<TX>, stream: &mut RlpStream) {
    stream.begin_list(15);
    stream.append(&header.parent_hash);
    stream.append(&header.uncles_hash);
    stream.append(&header.author);
    stream.append(&header.state_root);
    stream.append(&header.transactions_root);
    stream.append(&header.receipts_root);
    stream.append(&header.logs_bloom);
    stream.append(&header.difficulty);
    stream.append(&header.number.unwrap());
    stream.append(&header.gas_limit);
    stream.append(&header.gas_used);
    stream.append(&header.timestamp);
    stream.append(&header.extra_data.0);
    stream.append(&header.mix_hash.unwrap());
    stream.append(&header.nonce.unwrap());
}

fn get_context(input: Vec<u8>) -> VMContext {
    VMContext {
        current_account_id: "alice.near".to_string(),
        signer_account_id: "bob.near".to_string(),
        signer_account_pk: vec![0, 1, 2],
        predecessor_account_id: "carol.near".to_string(),
        input,
        block_index: 0,
        account_balance: 0,
        storage_usage: 0,
        attached_deposit: 0,
        prepaid_gas: 10u64.pow(9),
        random_seed: vec![0, 1, 2],
        free_of_charge: false,
        output_data_receivers: vec![],
    }
}

fn get_web3() -> web3::Web3<web3::transports::Http> {
    let (eloop, transport) = web3::transports::Http::new("https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2").unwrap();
    eloop.into_remote();
    web3::Web3::new(transport)
}

#[cfg(feature = "env_test")]
#[cfg(test)]
#[test]
fn add_block_headers() {
    let context = get_context(vec![]);
    let config = Config::default();
    testing_env!(context, config);

    let web3rust = get_web3();

    // Check on 400000 block from this answer: https://ethereum.stackexchange.com/a/67333/3032
    let block = web3rust.eth().block(BlockId::Number(BlockNumber::Number(400000.into()))).wait().unwrap().unwrap();

    let mut stream = RlpStream::new();
    rlp_append(&block, &mut stream);
    let out = stream.out();

    // println!("{:x?}", out.as_slice());

    let mut contract = EthBridge::default();
    contract.add_block_headers(400000, vec![out]);
    assert_eq!(block.hash.unwrap(), contract.block_hash(400000).unwrap().into());
}

// #[cfg(feature = "env_test")]
// #[cfg(test)]
// #[test]
// fn get_nonexistent_message() {
//     let context = get_context(vec![]);
//     let config = Config::default();
//     testing_env!(context, config);

//     // let contract = EthBridge::default();
//     // assert_eq!(None, contract.get_status("francis.near".to_string()));
// }
