extern crate web3;

use eth_bridge::{EthBridge,types::H128,header::NodeWithMerkleProof};
use web3::futures::Future;
use web3::types::{H256, Block};
use rlp::{RlpStream};
use futures::future::{join_all};
use std::panic;
use ethereum_types;
use serde::{Deserialize,Deserializer};
use hex::{FromHex, ToHex};

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate hex_literal;

fn catch_unwind_silent<F: FnOnce() -> R + panic::UnwindSafe, R>(f: F) -> std::thread::Result<R> {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(f);
    panic::set_hook(prev_hook);
    result
}

#[derive(Debug)]
struct Hex(Vec<u8>);

#[derive(Debug, Deserialize)]
//#[serde(rename_all = "PascalCase")]
struct DagMerkleRoot {
    pub dag_merkle_roots: Vec<H128>,
}

#[derive(Debug, Deserialize)]
//#[serde(rename_all = "PascalCase")]
struct BlockWithProofs {
    pub proof_length: u64,
    pub header_rlp: Hex,
    pub merkle_root: H128,
    pub elements: Vec<H256>,
    pub merkle_proofs: Vec<H128>,
}

impl<'de> Deserialize<'de> for Hex {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
        where
            D: Deserializer<'de>,
    {
        let mut s = String::deserialize(deserializer)?;
        if s.starts_with("0x") {
            s = s[2..].to_string();
        }
        if s.len() % 2 == 1 {
            s.insert_str(0, "0");
        }
        Ok(Hex(Vec::from_hex(&s).map_err(|err| serde::de::Error::custom(err.to_string()))?))
    }
}

// Wish to avoid this code and use web3+rlp libraries directly
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

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_bindgen::MockedBlockchain;
    use near_bindgen::{testing_env, VMContext};

    lazy_static! {
        static ref WEB3RS: web3::Web3<web3::transports::Http> = {
            let (eloop, transport) = web3::transports::Http::new("https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2").unwrap();
            eloop.into_remote();
            web3::Web3::new(transport)
        };
    }

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice.near".to_string(),
            signer_account_id: "bob.near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol.near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(9),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
        }
    }

    fn get_blocks(web3rust: &web3::Web3<web3::transports::Http>, start: usize, stop: usize)
        -> (Vec<H256>, Vec<Vec<u8>>, Vec<H64>)
    {

        let futures = (start..stop).map(
            |i| web3rust.eth().block((i as u64).into())
        ).collect::<Vec<_>>();

        let block_headers = join_all(futures).wait().unwrap();

        let mut hashes: Vec<H256> = vec![];
        let mut blocks: Vec<Vec<u8>> = vec![];
        let mut nonces: Vec<H64> = vec![];
        for block_header in block_headers {
            let mut stream = RlpStream::new();
            rlp_append(&block_header.clone().unwrap(), &mut stream);
            hashes.push(block_header.clone().unwrap().hash.unwrap());
            blocks.push(stream.out());
            nonces.push(block_headerclone().unwrap().nonce.unwrap())
        }

        (hashes, blocks, nonces)
    }

    #[test]
    fn add_dags_merkle_roots() {
        testing_env!(get_context(vec![], false));

        let dmr: DagMerkleRoot = serde_json::from_reader(
            std::fs::File::open(std::path::Path::new("./tests/dag_merkle_roots.json")).unwrap()
        ).unwrap();

        let mut contract = EthBridge::default();
        contract.init(0, dmr.dag_merkle_roots.clone());
        assert_eq!(dmr.dag_merkle_roots[0], contract.dag_merkle_root(0));
        assert_eq!(dmr.dag_merkle_roots[10], contract.dag_merkle_root(10));
        assert_eq!(dmr.dag_merkle_roots[511], contract.dag_merkle_root(511));

        let result = catch_unwind_silent(|| contract.dag_merkle_root(512));
        assert!(result.is_err());
    }

    #[test]
    fn add_400000_block_only() {
        testing_env!(get_context(vec![], false));

        // Check on 400000 block from this answer: https://ethereum.stackexchange.com/a/67333/3032
        let (hashes, blocks) = get_blocks(&WEB3RS, 400_000, 400_001);

        // $ ../ethrelay/ethashproof/cmd/relayer/relayer 400000
        // digest: 0x3fbea7af642a4e20cd93a945a1f5e23bd72fc5261153e09102cf718980aeff38
        // ethash result: 0x00000000000ca599ebe9913fa00da78a4d1dd2fa154c4fd2aad10ccbca52a2a1
        // Proof length: 24
        // [400000.json]

        let block: BlockWithProofs = serde_json::from_reader(
            std::fs::File::open(std::path::Path::new("./tests/400000.json")).unwrap()
        ).unwrap();

        let mut contract = EthBridge::default();
        contract.init(400_000 / 30000, vec![block.merkle_root]);
        let result = catch_unwind_silent(panic::AssertUnwindSafe(
            || contract.add_block_headers(
                blocks.clone(),
                blocks.iter().map(|b| b.nonce()).collect(),
                vec![NodeWithMerkleProof(
                    
                    block.merkle_proofs
                )]
            )
        ));
        assert!(result.is_err());
        contract.add_block_headers(400_000, blocks);
        assert_eq!(hashes[0], (contract.block_hash_unsafe(400_000).unwrap().0).0.into());
    }

    // #[test]
    // fn add_20_blocks_from_8000000() {
    //     testing_env!(get_context(vec![], false));

    //     let start: usize = 8_000_000;
    //     let stop: usize = 8_000_020;

    //     let (hashes, blocks) = get_blocks(&WEB3RS, start, stop);
        
    //     let mut contract = EthBridge::default();
    //     contract.add_block_headers(start as u64, blocks);

    //     for i in start..stop {
    //         assert_eq!(hashes[i - start], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    // }

    // #[test]
    // fn add_3_sequential_ranges_of_blocks() {
    //     testing_env!(get_context(vec![], false));

    //     let (hashes1, blocks1) = get_blocks(&WEB3RS, 8_000_000, 8_000_010);
    //     let (hashes2, blocks2) = get_blocks(&WEB3RS, 8_000_010, 8_000_020);
    //     let (hashes3, blocks3) = get_blocks(&WEB3RS, 8_000_020, 8_000_030);
        
    //     let mut contract = EthBridge::default();
    //     contract.add_block_headers(8_000_000 as u64, blocks1);
    //     contract.add_block_headers(8_000_010 as u64, blocks2);
    //     contract.add_block_headers(8_000_020 as u64, blocks3);

    //     for i in 8_000_000..8_000_010 {
    //         assert_eq!(hashes1[i - 8_000_000], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    //     for i in 8_000_010..8_000_020 {
    //         assert_eq!(hashes2[i - 8_000_010], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    //     for i in 8_000_020..8_000_030 {
    //         assert_eq!(hashes3[i - 8_000_020], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    // }

    // #[test]
    // fn add_3_intersecting_ranges_of_blocks() {
    //     testing_env!(get_context(vec![], false));

    //     let (hashes1, blocks1) = get_blocks(&WEB3RS, 8_000_000, 8_000_010);
    //     let (hashes2, blocks2) = get_blocks(&WEB3RS, 8_000_005, 8_000_020);
    //     let (hashes3, blocks3) = get_blocks(&WEB3RS, 8_000_015, 8_000_030);
        
    //     let mut contract = EthBridge::default();
    //     contract.add_block_headers(8_000_000 as u64, blocks1);
    //     contract.add_block_headers(8_000_005 as u64, blocks2);
    //     contract.add_block_headers(8_000_015 as u64, blocks3);

    //     for i in 8_000_000..8_000_010 {
    //         assert_eq!(hashes1[i - 8_000_000], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    //     for i in 8_000_005..8_000_020 {
    //         assert_eq!(hashes2[i - 8_000_005], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    //     for i in 8_000_015..8_000_030 {
    //         assert_eq!(hashes3[i - 8_000_015], (contract.block_hash_unsafe(i as u64).unwrap().0).0.into());
    //     }
    // }
}
