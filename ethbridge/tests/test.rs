extern crate web3;

use eth_bridge::{EthBridge};
use near_bindgen::MockedBlockchain;
use near_bindgen::{VMContext, Config, testing_env};
use web3::futures::Future;
use web3::types::{H256, BlockId, BlockNumber, Block};
use rlp::{RlpStream};
use futures::future::{join_all};

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

fn get_blocks(web3rust: &web3::Web3<web3::transports::Http>, start: usize, stop: usize) -> (Vec<H256>, Vec<Vec<u8>>) {

    let futures = (start..stop).map(|i| web3rust.eth().block(BlockId::Number(BlockNumber::Number(i.into())))).collect::<Vec<_>>();

    let block_headers = join_all(futures).wait().unwrap();

    let mut hashes: Vec<H256> = vec![];
    let mut blocks: Vec<Vec<u8>> = vec![];
    for block_header in block_headers {
        let mut stream = RlpStream::new();
        rlp_append(&block_header.clone().unwrap(), &mut stream);
        hashes.push(block_header.clone().unwrap().hash.unwrap());
        blocks.push(stream.out());
    }

    (hashes, blocks)
}

#[cfg(feature = "env_test")]
#[cfg(test)]
#[test]
fn add_400000_block_only() {
    let context = get_context(vec![]);
    let config = Config::default();
    testing_env!(context, config);

    let web3rust = get_web3();

    // Check on 400000 block from this answer: https://ethereum.stackexchange.com/a/67333/3032
    let (hashes, blocks) = get_blocks(&web3rust, 400_000, 400_001);

    let mut contract = EthBridge::default();
    contract.add_block_headers(400_000, blocks);
    assert_eq!(hashes[0], contract.block_hash_unsafe(400_000).unwrap().into());
}

#[cfg(feature = "env_test")]
#[cfg(test)]
#[test]
fn add_20_blocks_from_8000000() {
    let context = get_context(vec![]);
    let config = Config::default();
    testing_env!(context, config);

    let start: usize = 8_000_000;
    let stop: usize = 8_000_020;
    let web3rust = get_web3();

    let (hashes, blocks) = get_blocks(&web3rust, start, stop);
    
    let mut contract = EthBridge::default();
    contract.add_block_headers(start as u64, blocks);

    for i in start..stop {
        assert_eq!(hashes[i - start], contract.block_hash_unsafe(i as u64).unwrap().into());
    }
}

#[cfg(feature = "env_test")]
#[cfg(test)]
#[test]
fn add_3_sequential_ranges_of_blocks() {
    let context = get_context(vec![]);
    let config = Config::default();
    testing_env!(context, config);

    let web3rust = get_web3();

    let (hashes1, blocks1) = get_blocks(&web3rust, 8_000_000, 8_000_010);
    let (hashes2, blocks2) = get_blocks(&web3rust, 8_000_010, 8_000_020);
    let (hashes3, blocks3) = get_blocks(&web3rust, 8_000_020, 8_000_030);
    
    let mut contract = EthBridge::default();
    contract.add_block_headers(8_000_000 as u64, blocks1);
    contract.add_block_headers(8_000_010 as u64, blocks2);
    contract.add_block_headers(8_000_020 as u64, blocks3);

    for i in 8_000_000..8_000_010 {
        assert_eq!(hashes1[i - 8_000_000], contract.block_hash_unsafe(i as u64).unwrap().into());
    }
    for i in 8_000_010..8_000_020 {
        assert_eq!(hashes2[i - 8_000_010], contract.block_hash_unsafe(i as u64).unwrap().into());
    }
    for i in 8_000_020..8_000_030 {
        assert_eq!(hashes3[i - 8_000_020], contract.block_hash_unsafe(i as u64).unwrap().into());
    }
}

#[cfg(feature = "env_test")]
#[cfg(test)]
#[test]
fn add_3_intersecting_ranges_of_blocks() {
    let context = get_context(vec![]);
    let config = Config::default();
    testing_env!(context, config);

    let web3rust = get_web3();

    let (hashes1, blocks1) = get_blocks(&web3rust, 8_000_000, 8_000_010);
    let (hashes2, blocks2) = get_blocks(&web3rust, 8_000_005, 8_000_020);
    let (hashes3, blocks3) = get_blocks(&web3rust, 8_000_015, 8_000_030);
    
    let mut contract = EthBridge::default();
    contract.add_block_headers(8_000_000 as u64, blocks1);
    contract.add_block_headers(8_000_005 as u64, blocks2);
    contract.add_block_headers(8_000_015 as u64, blocks3);

    for i in 8_000_000..8_000_010 {
        assert_eq!(hashes1[i - 8_000_000], contract.block_hash_unsafe(i as u64).unwrap().into());
    }
    for i in 8_000_005..8_000_020 {
        assert_eq!(hashes2[i - 8_000_005], contract.block_hash_unsafe(i as u64).unwrap().into());
    }
    for i in 8_000_015..8_000_030 {
        assert_eq!(hashes3[i - 8_000_015], contract.block_hash_unsafe(i as u64).unwrap().into());
    }
}
