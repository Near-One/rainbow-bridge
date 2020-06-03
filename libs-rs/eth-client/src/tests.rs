use futures::future::join_all;
use std::panic;

use crate::{DoubleNodeWithMerkleProof, EthBridge};
use eth_types::*;
use hex::FromHex;
use rlp::RlpStream;
use serde::{Deserialize, Deserializer};
use web3::futures::Future;
use web3::types::Block;

//#[macro_use]
//extern crate lazy_static;
use lazy_static::lazy_static;

fn catch_unwind_silent<F: FnOnce() -> R + panic::UnwindSafe, R>(f: F) -> std::thread::Result<R> {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(f);
    panic::set_hook(prev_hook);
    result
}

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

#[derive(Debug, Deserialize)]
struct RootsCollectionRaw {
    pub dag_merkle_roots: Vec<Hex>, // H128
}

#[derive(Debug, Deserialize)]
struct RootsCollection {
    pub dag_merkle_roots: Vec<H128>,
}

impl From<RootsCollectionRaw> for RootsCollection {
    fn from(item: RootsCollectionRaw) -> Self {
        Self {
            dag_merkle_roots: item
                .dag_merkle_roots
                .iter()
                .map(|e| H128::from(&e.0))
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct BlockWithProofsRaw {
    pub proof_length: u64,
    pub header_rlp: Hex,
    pub merkle_root: Hex,        // H128
    pub elements: Vec<Hex>,      // H256
    pub merkle_proofs: Vec<Hex>, // H128
}

#[derive(Debug, Deserialize)]
struct BlockWithProofs {
    pub proof_length: u64,
    pub header_rlp: Hex,
    pub merkle_root: H128,
    pub elements: Vec<H256>,
    pub merkle_proofs: Vec<H128>,
}

impl From<BlockWithProofsRaw> for BlockWithProofs {
    fn from(item: BlockWithProofsRaw) -> Self {
        Self {
            proof_length: item.proof_length,
            header_rlp: item.header_rlp,
            merkle_root: H128::from(&item.merkle_root.0),
            elements: item.elements.iter().map(|e| H256::from(&e.0)).collect(),
            merkle_proofs: item
                .merkle_proofs
                .iter()
                .map(|e| H128::from(&e.0))
                .collect(),
        }
    }
}

impl BlockWithProofs {
    fn combine_dag_h256_to_h512(elements: Vec<H256>) -> Vec<H512> {
        elements
            .iter()
            .zip(elements.iter().skip(1))
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, (a, b))| {
                let mut buffer = [0u8; 64];
                buffer[..32].copy_from_slice(&(a.0).0);
                buffer[32..].copy_from_slice(&(b.0).0);
                H512(buffer.into())
            })
            .collect()
    }

    pub fn to_double_node_with_merkle_proof_vec(&self) -> Vec<DoubleNodeWithMerkleProof> {
        let h512s = Self::combine_dag_h256_to_h512(self.elements.clone());
        h512s
            .iter()
            .zip(h512s.iter().skip(1))
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(i, (a, b))| DoubleNodeWithMerkleProof {
                dag_nodes: vec![*a, *b],
                proof: self.merkle_proofs
                    [i / 2 * self.proof_length as usize..(i / 2 + 1) * self.proof_length as usize]
                    .to_vec(),
            })
            .collect()
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

// TESTS

use near_bindgen::MockedBlockchain;
use near_bindgen::{testing_env, VMContext};

lazy_static! {
    static ref WEB3RS: web3::Web3<web3::transports::Http> = {
        let (eloop, transport) = web3::transports::Http::new(
            "https://mainnet.infura.io/v3/b5f870422ee5454fb11937e947154cd2",
        )
        .unwrap();
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
        prepaid_gas: 10u64.pow(18),
        random_seed: vec![0, 1, 2],
        is_view,
        output_data_receivers: vec![],
    }
}

fn get_blocks(
    web3rust: &web3::Web3<web3::transports::Http>,
    start: usize,
    stop: usize,
) -> (Vec<Vec<u8>>, Vec<H256>) {
    let futures = (start..stop)
        .map(|i| web3rust.eth().block((i as u64).into()))
        .collect::<Vec<_>>();

    let block_headers = join_all(futures).wait().unwrap();

    let mut blocks: Vec<Vec<u8>> = vec![];
    let mut hashes: Vec<H256> = vec![];
    for block_header in block_headers {
        let mut stream = RlpStream::new();
        rlp_append(&block_header.clone().unwrap(), &mut stream);
        blocks.push(stream.out());
        hashes.push(H256(block_header.clone().unwrap().hash.unwrap().0.into()));
    }

    (blocks, hashes)
}

fn read_roots_collection() -> RootsCollection {
    read_roots_collection_raw().into()
}

fn read_roots_collection_raw() -> RootsCollectionRaw {
    serde_json::from_reader(
        std::fs::File::open(std::path::Path::new("./src/data/dag_merkle_roots.json")).unwrap(),
    )
    .unwrap()
}

fn read_block(filename: String) -> BlockWithProofs {
    read_block_raw(filename).into()
}

fn read_block_raw(filename: String) -> BlockWithProofsRaw {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(&filename)).unwrap()).unwrap()
}

#[test]
fn add_dags_merkle_roots() {
    testing_env!(get_context(vec![], false));

    let dmr = read_roots_collection();
    let contract = EthBridge::init(true, 0, read_roots_collection().dag_merkle_roots);

    assert_eq!(dmr.dag_merkle_roots[0], contract.dag_merkle_root(0));
    assert_eq!(dmr.dag_merkle_roots[10], contract.dag_merkle_root(10));
    assert_eq!(dmr.dag_merkle_roots[511], contract.dag_merkle_root(511));

    let result = catch_unwind_silent(|| contract.dag_merkle_root(512));
    assert!(result.is_err());
}

#[test]
fn add_blocks_2_and_3() {
    testing_env!(get_context(vec![], false));

    // Check on 3 block from here: https://github.com/KyberNetwork/bridge_eos_smart_contracts/blob/master/scripts/jungle/jungle_relay_3.js
    let (blocks, hashes) = get_blocks(&WEB3RS, 2, 4);

    // $ ../ethrelay/ethashproof/cmd/relayer/relayer 3
    let blocks_with_proofs: Vec<BlockWithProofs> = ["./src/data/2.json", "./src/data/3.json"]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();

    let mut contract = EthBridge::init(true, 0, read_roots_collection().dag_merkle_roots);

    for (block, proof) in blocks.into_iter().zip(blocks_with_proofs.into_iter()) {
        contract.add_block_header(block, proof.to_double_node_with_merkle_proof_vec());
    }

    assert_eq!((hashes[1].0).0, (contract.block_hash(3).unwrap().0).0);
}

#[test]
fn add_400000_block_only() {
    testing_env!(get_context(vec![], false));

    // Check on 400000 block from this answer: https://ethereum.stackexchange.com/a/67333/3032
    let (blocks, hashes) = get_blocks(&WEB3RS, 400_000, 400_001);

    // $ ../ethrelay/ethashproof/cmd/relayer/relayer 400000
    // digest: 0x3fbea7af642a4e20cd93a945a1f5e23bd72fc5261153e09102cf718980aeff38
    // ethash result: 0x00000000000ca599ebe9913fa00da78a4d1dd2fa154c4fd2aad10ccbca52a2a1
    // Proof length: 24
    // [400000.json]

    let block_with_proof = read_block("./src/data/400000.json".to_string());

    let mut contract = EthBridge::init(true, 400_000 / 30000, vec![block_with_proof.merkle_root]);

    // let result = catch_unwind_silent(panic::AssertUnwindSafe(
    //     || contract.add_block_headers(
    //         blocks,
    //         nonces.iter().map(|n| H64(n.0)).collect::<Vec<H64>>(),
    //         vec![{
    //             let h512s = combine_dag_h256_to_h512(block.elements);
    //             h512s.iter().zip(h512s.iter().skip(1)).filter(|(i,_)| {
    //                 i % 2 == 1
    //             }).map(|(_,(a,b))| {
    //                 DoubleNodeWithMerkleProof {
    //                     dag_nodes: vec![*a, *b],
    //                     proof: block.merkle_proofs,
    //                 }
    //             }).collect()
    //         }]
    //     )
    // ));
    // assert!(result.is_err());

    contract.add_block_header(
        blocks.into_iter().next().unwrap(),
        block_with_proof.to_double_node_with_merkle_proof_vec(),
    );
    assert_eq!((hashes[0].0).0, (contract.block_hash(400_000).unwrap().0).0);
}

#[test]
fn add_two_blocks_from_8996776() {
    testing_env!(get_context(vec![], false));

    // Check on 8996777 block from this test: https://github.com/sorpaas/rust-ethash/blob/ac6e42bcb7f40ad2a3b89f7400a61f7baf3f0926/src/lib.rs#L318-L326
    let (blocks, hashes) = get_blocks(&WEB3RS, 8_996_776, 8_996_778);

    // $ ../ethrelay/ethashproof/cmd/relayer/relayer 8996777
    let blocks_with_proofs: Vec<BlockWithProofs> =
        ["./src/data/8996776.json", "./src/data/8996777.json"]
            .iter()
            .map(|filename| read_block((&filename).to_string()))
            .collect();

    let mut contract = EthBridge::init(true, 0, read_roots_collection().dag_merkle_roots);

    for (block, proof) in blocks.into_iter().zip(blocks_with_proofs.into_iter()) {
        contract.add_block_header(block, proof.to_double_node_with_merkle_proof_vec());
    }

    assert_eq!(
        (hashes[0].0).0,
        (contract.block_hash(8_996_776).unwrap().0).0
    );
    assert_eq!(
        (hashes[1].0).0,
        (contract.block_hash(8_996_777).unwrap().0).0
    );
}

#[test]
fn add_2_blocks_from_400000() {
    testing_env!(get_context(vec![], false));

    // Check on 400000 block from this answer: https://ethereum.stackexchange.com/a/67333/3032
    let (blocks, hashes) = get_blocks(&WEB3RS, 400_000, 400_002);

    // $ ../ethrelay/ethashproof/cmd/relayer/relayer 400001
    // digest: 0x3fbea7af642a4e20cd93a945a1f5e23bd72fc5261153e09102cf718980aeff38
    // ethash result: 0x00000000000ca599ebe9913fa00da78a4d1dd2fa154c4fd2aad10ccbca52a2a1
    // Proof length: 24
    // [400001.json]

    let blocks_with_proofs: Vec<BlockWithProofs> =
        ["./src/data/400000.json", "./src/data/400001.json"]
            .iter()
            .map(|filename| read_block((&filename).to_string()))
            .collect();

    let mut contract = EthBridge::init(
        true,
        400_000 / 30000,
        vec![blocks_with_proofs.first().unwrap().merkle_root],
    );

    for (block, proof) in blocks.into_iter().zip(blocks_with_proofs.into_iter()) {
        contract.add_block_header(block, proof.to_double_node_with_merkle_proof_vec());
    }
    assert_eq!((hashes[0].0).0, (contract.block_hash(400_000).unwrap().0).0);
    assert_eq!((hashes[1].0).0, (contract.block_hash(400_001).unwrap().0).0);
}

// #[test]
// fn add_3_sequential_ranges_of_blocks() {
//     testing_env!(get_context(vec![], false));
//
//     let (hashes1, blocks1) = get_blocks(&WEB3RS, 8_000_000, 8_000_011);
//     let (hashes2, blocks2) = get_blocks(&WEB3RS, 8_000_010, 8_000_021);
//     let (hashes3, blocks3) = get_blocks(&WEB3RS, 8_000_020, 8_000_031);
//
//     let blocks_with_proofs: Vec<BlockWithProofs> = [
//         "./src/data/8000000.json",
//         "./src/data/8000001.json"
//     ].iter().map(|filename| read_block((&filename).to_string())).collect();
//
//     let mut contract = EthBridge::default();
//     contract.init(true, 0, read_roots_collection().dag_merkle_roots);
//
//     contract.add_block_headers(blocks1);
//     contract.add_block_headers(blocks2);
//     contract.add_block_headers(blocks3);
//
//     for i in 8_000_000..8_000_010 {
//         assert_eq!(hashes1[i - 8_000_000], (contract.block_hash(i as u64).unwrap().0).0.into());
//     }
//     for i in 8_000_010..8_000_020 {
//         assert_eq!(hashes2[i - 8_000_010], (contract.block_hash(i as u64).unwrap().0).0.into());
//     }
//     for i in 8_000_020..8_000_030 {
//         assert_eq!(hashes3[i - 8_000_020], (contract.block_hash(i as u64).unwrap().0).0.into());
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
//         assert_eq!(hashes1[i - 8_000_000], (contract.block_hash(i as u64).unwrap().0).0.into());
//     }
//     for i in 8_000_005..8_000_020 {
//         assert_eq!(hashes2[i - 8_000_005], (contract.block_hash(i as u64).unwrap().0).0.into());
//     }
//     for i in 8_000_015..8_000_030 {
//         assert_eq!(hashes3[i - 8_000_015], (contract.block_hash(i as u64).unwrap().0).0.into());
//     }
// }

#[test]
fn predumped_block_can_be_added() {
    use std::env;
    use std::fs;

    testing_env!(get_context(vec![], false));
    
    let mut blocks_with_proofs = fs::read_dir(env::var("ETH_HEADER_DIR").unwrap())
        .unwrap()
        .map(|path| path.unwrap().path().display().to_string())
        .map(|s| {
            (
                s.clone()
                    .split('/')
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
                    .split('.')
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
                s,
            )
        })
        .collect::<Vec<_>>();
    blocks_with_proofs.sort_by_key(|s| s.0);
    let start_block_height = blocks_with_proofs.first().unwrap().0;
    let blocks_with_proofs: Vec<_> = blocks_with_proofs
        .iter()
        .map(|filename| read_block(filename.1.to_string()))
        .collect();

    let mut contract = EthBridge::init(
        true,
        start_block_height / 30000,
        vec![blocks_with_proofs.first().unwrap().merkle_root],
    );

    for block_with_proof in blocks_with_proofs.into_iter() {
        contract.add_block_header(
            block_with_proof.header_rlp.0.clone(),
            block_with_proof.to_double_node_with_merkle_proof_vec(),
        );
    }
}
