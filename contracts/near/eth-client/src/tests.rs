use std::panic;

use crate::{DoubleNodeWithMerkleProof, EthClient};
use eth_types::*;
use hex::FromHex;
use rlp::RlpStream;
use serde::{Deserialize, Deserializer};
use web3::types::Block;

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
    stream.append(&header.logs_bloom.unwrap());
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

use near_sdk::{testing_env, VMContext};

lazy_static! {
    static ref WEB3RS: web3::Web3<web3::transports::Http> = {
        let transport = web3::transports::Http::new(
            format!(
                "https://mainnet.infura.io/v3/{}",
                std::env::var("ETH1_INFURA_API_KEY").unwrap()
            )
            .as_str(),
        )
        .unwrap();
        web3::Web3::new(transport)
    };
}

fn get_context() -> VMContext {
    VMContext {
        current_account_id: "alice.near".parse().unwrap(),
        signer_account_id: "bob.near".parse().unwrap(),
        signer_account_pk: "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp"
            .parse()
            .unwrap(),
        predecessor_account_id: "carol.near".parse().unwrap(),
        input: vec![],
        block_index: 0,
        block_timestamp: 0,
        account_balance: 0,
        account_locked_balance: 0,
        epoch_height: 0,
        storage_usage: 0,
        attached_deposit: 0,
        prepaid_gas: near_sdk::Gas(10u64.pow(18)),
        random_seed: vec![1; 32].try_into().unwrap(),
        view_config: None,
        output_data_receivers: vec![],
    }
}

fn get_blocks(
    web3rust: &web3::Web3<web3::transports::Http>,
    start: usize,
    stop: usize,
) -> (Vec<Vec<u8>>, Vec<H256>) {
    let futures = (start..stop)
        .map(|i| {
            web3rust.eth().block(web3::types::BlockId::Number(
                web3::types::BlockNumber::Number(i.into()),
            ))
        })
        .collect::<Vec<_>>();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let block_headers = rt.block_on(web3::futures::future::join_all(futures));

    let mut blocks: Vec<Vec<u8>> = vec![];
    let mut hashes: Vec<H256> = vec![];
    for block_header in block_headers {
        let mut stream = RlpStream::new();
        rlp_append(&block_header.clone().unwrap().unwrap(), &mut stream);
        blocks.push(stream.out().to_vec());
        hashes.push(H256(block_header.unwrap().unwrap().hash.unwrap().0.into()));
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

fn assert_hashes_equal_to_contract_hashes(
    contract: &EthClient,
    heights: &[u64],
    real_hashes: &[H256],
) {
    let hashes_from_contract: Vec<H256> = heights
        .iter()
        .map(|height| contract.block_hash(*height).unwrap())
        .collect();

    for (hash, hash_from_contract) in real_hashes.into_iter().zip(hashes_from_contract.iter()) {
        assert_eq!(hash, hash_from_contract);
    }
}

#[test]
fn add_dags_merkle_roots() {
    testing_env!(get_context());
    let (blocks, _) = get_blocks(&WEB3RS, 400_000, 400_001);

    let dmr = read_roots_collection();
    let contract = EthClient::init(
        true,
        0,
        read_roots_collection().dag_merkle_roots,
        blocks[0].clone(),
        30,
        10,
        10,
        None,
    );

    for i in 0..699 {
        assert_eq!(dmr.dag_merkle_roots[i], contract.dag_merkle_root(i as u64));
    }

    // Get non-existing DAG Merkle root
    let result = catch_unwind_silent(|| contract.dag_merkle_root(700));
    assert!(result.is_err());
}

#[test]
fn update_dags_merkle_roots() {
    let block = read_block(format!("./src/data/{}.json", 12_965_000).to_string());
    let mut context = get_context();
    context.predecessor_account_id = context.current_account_id.clone();
    testing_env!(context.clone());

    let dmr = read_roots_collection();
    let mut contract = EthClient::init(
        true,
        0,
        read_roots_collection().dag_merkle_roots,
        block.header_rlp.0,
        30,
        10,
        10,
        None,
    );

    contract.update_dags_merkle_roots(0, dmr.dag_merkle_roots.clone());

    for i in 0..699 {
        assert_eq!(dmr.dag_merkle_roots[i], contract.dag_merkle_root(i as u64));
    }

    // Get non-existing DAG Merkle root
    let result = catch_unwind_silent(|| contract.dag_merkle_root(700));
    assert!(result.is_err());

    // Test with the starting offset for DAG Merkle roots
    let start_epoch: usize = 490;
    contract.update_dags_merkle_roots(
        start_epoch as u64,
        dmr.dag_merkle_roots[start_epoch..].to_vec(),
    );
    for i in start_epoch..699 {
        assert_eq!(dmr.dag_merkle_roots[i], contract.dag_merkle_root(i as u64));
    }
    let result = catch_unwind_silent(|| contract.dag_merkle_root((start_epoch - 1) as u64));
    assert!(result.is_err());
}

#[test]
fn add_blocks_2_and_3() {
    testing_env!(get_context());

    // Check on 3 block from here: https://github.com/KyberNetwork/bridge_eos_smart_contracts/blob/master/scripts/jungle/jungle_relay_3.js
    let (blocks, hashes) = get_blocks(&WEB3RS, 1, 4);

    // $ ../ethrelay/ethashproof/cmd/relayer/relayer 3
    let blocks_with_proofs: Vec<BlockWithProofs> = ["./src/data/2.json", "./src/data/3.json"]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();

    let mut contract = EthClient::init(
        true,
        0,
        read_roots_collection().dag_merkle_roots,
        blocks[0].clone(),
        30,
        10,
        10,
        None,
    );

    for (block, proof) in blocks
        .into_iter()
        .skip(1) // Skip parent header
        .zip(blocks_with_proofs.into_iter())
    {
        contract.add_block_header(block, proof.to_double_node_with_merkle_proof_vec());
    }

    let heights = [2, 3];
    // Skip parent header hash
    let hashes = &hashes[1..];
    assert_hashes_equal_to_contract_hashes(&contract, &heights, &hashes);
}

#[test]
fn add_blocks_before_and_after_istanbul_fork() {
    testing_env!(get_context());

    const FORK_HEIGHT_ISTANBUL: usize = 9_069_000;
    let (blocks, hashes) = get_blocks(&WEB3RS, FORK_HEIGHT_ISTANBUL - 2, FORK_HEIGHT_ISTANBUL + 2);

    let blocks_with_proofs: Vec<BlockWithProofs> = [
        format!("./src/data/proof_block_{}.json", FORK_HEIGHT_ISTANBUL - 1),
        format!("./src/data/proof_block_{}.json", FORK_HEIGHT_ISTANBUL),
        format!("./src/data/proof_block_{}.json", FORK_HEIGHT_ISTANBUL + 1),
    ]
    .iter()
    .map(|filename| read_block((&filename).to_string()))
    .collect();

    let mut contract = EthClient::init(
        true,
        0,
        read_roots_collection().dag_merkle_roots,
        blocks[0].clone(),
        30,
        10,
        10,
        None,
    );

    for (block, proof) in blocks
        .into_iter()
        .skip(1) // Skip parent header
        .zip(blocks_with_proofs.into_iter())
    {
        contract.add_block_header(block, proof.to_double_node_with_merkle_proof_vec());
    }

    let heights = [
        FORK_HEIGHT_ISTANBUL as u64 - 1,
        FORK_HEIGHT_ISTANBUL as u64,
        FORK_HEIGHT_ISTANBUL as u64 + 1,
    ];
    // Skip parent header hash
    let hashes = &hashes[1..];
    assert_hashes_equal_to_contract_hashes(&contract, &heights, &hashes);
}

#[test]
fn add_blocks_before_and_after_nov11_2020_unannounced_fork() {
    testing_env!(get_context());

    const FORK_HEIGHT_UNANNOUNCED_NOV_11_2020: usize = 11_234_873;

    let (blocks, hashes) = get_blocks(
        &WEB3RS,
        FORK_HEIGHT_UNANNOUNCED_NOV_11_2020 - 2,
        FORK_HEIGHT_UNANNOUNCED_NOV_11_2020 + 2,
    );

    let blocks_with_proofs: Vec<BlockWithProofs> = [
        format!(
            "./src/data/proof_block_{}.json",
            FORK_HEIGHT_UNANNOUNCED_NOV_11_2020 - 1
        ),
        format!(
            "./src/data/proof_block_{}.json",
            FORK_HEIGHT_UNANNOUNCED_NOV_11_2020
        ),
        format!(
            "./src/data/proof_block_{}.json",
            FORK_HEIGHT_UNANNOUNCED_NOV_11_2020 + 1
        ),
    ]
    .iter()
    .map(|filename| read_block((&filename).to_string()))
    .collect();

    let mut contract = EthClient::init(
        true,
        0,
        read_roots_collection().dag_merkle_roots,
        blocks[0].clone(),
        30,
        10,
        10,
        None,
    );

    for (block, proof) in blocks
        .into_iter()
        .skip(1) // Skip parent header
        .zip(blocks_with_proofs.into_iter())
    {
        contract.add_block_header(block, proof.to_double_node_with_merkle_proof_vec());
    }
    let heights = [
        FORK_HEIGHT_UNANNOUNCED_NOV_11_2020 as u64 - 1,
        FORK_HEIGHT_UNANNOUNCED_NOV_11_2020 as u64,
        FORK_HEIGHT_UNANNOUNCED_NOV_11_2020 as u64 + 1,
    ];
    // Skip parent header hash
    let hashes = &hashes[1..];
    assert_hashes_equal_to_contract_hashes(&contract, &heights, &hashes);
}

#[test]
fn add_block_diverged_until_ethashproof_dataset_fix() {
    testing_env!(get_context());

    const HEIGHT_DIVERGED_BLOCK: usize = 11_703_828;
    let (blocks, hashes) = get_blocks(
        &WEB3RS,
        HEIGHT_DIVERGED_BLOCK - 1,
        HEIGHT_DIVERGED_BLOCK + 1,
    );
    // Jan 22 2021
    let block_with_proof = read_block(format!(
        "./src/data/proof_block_{}.json",
        HEIGHT_DIVERGED_BLOCK
    ));

    let mut contract = EthClient::init(
        true,
        0,
        read_roots_collection().dag_merkle_roots,
        blocks[0].clone(),
        90000,
        500,
        20,
        None,
    );

    contract.add_block_header(
        blocks[1].clone(),
        block_with_proof.to_double_node_with_merkle_proof_vec(),
    );
    assert_eq!(
        hashes[1],
        contract.block_hash(HEIGHT_DIVERGED_BLOCK as u64).unwrap()
    );
}

#[test]
fn add_400000_block_only() {
    testing_env!(get_context());

    // Check on 400000 block from this answer: https://ethereum.stackexchange.com/a/67333/3032
    let block_height = 400_000;
    let (blocks, hashes) = get_blocks(&WEB3RS, block_height - 1, block_height + 1);

    // $ ../ethrelay/ethashproof/cmd/relayer/relayer 400000
    // digest: 0x3fbea7af642a4e20cd93a945a1f5e23bd72fc5261153e09102cf718980aeff38
    // ethash result: 0x00000000000ca599ebe9913fa00da78a4d1dd2fa154c4fd2aad10ccbca52a2a1
    // Proof length: 24
    // [400000.json]

    let block_with_proof = read_block(format!("./src/data/{}.json", block_height));
    let mut contract = EthClient::init(
        true,
        400_000 / 30000,
        vec![block_with_proof.merkle_root],
        blocks[0].clone(),
        30,
        10,
        10,
        None,
    );
    contract.add_block_header(
        blocks[1].clone(),
        block_with_proof.to_double_node_with_merkle_proof_vec(),
    );
    assert_eq!(hashes[1], contract.block_hash(block_height as u64).unwrap());
}

#[test]
fn add_two_blocks_from_8996776() {
    testing_env!(get_context());

    // Check on 8996777 block from this test: https://github.com/sorpaas/rust-ethash/blob/ac6e42bcb7f40ad2a3b89f7400a61f7baf3f0926/src/lib.rs#L318-L326
    let block_height = 8_996_776;
    let (blocks, hashes) = get_blocks(&WEB3RS, block_height - 1, block_height + 2);

    // $ ../ethrelay/ethashproof/cmd/relayer/relayer 8996777
    let blocks_with_proofs: Vec<BlockWithProofs> = [
        format!("./src/data/{}.json", block_height),
        format!("./src/data/{}.json", block_height + 1),
    ]
    .iter()
    .map(|filename| read_block((&filename).to_string()))
    .collect();

    let mut contract = EthClient::init(
        true,
        0,
        read_roots_collection().dag_merkle_roots,
        blocks[0].clone(),
        30,
        10,
        10,
        None,
    );

    for (block, proof) in blocks
        .into_iter()
        .skip(1)
        .zip(blocks_with_proofs.into_iter())
    {
        contract.add_block_header(block, proof.to_double_node_with_merkle_proof_vec());
    }

    let heights = [block_height as u64, block_height as u64 + 1];
    // Skip parent header hash
    let hashes = &hashes[1..];
    assert_hashes_equal_to_contract_hashes(&contract, &heights, &hashes);
}

#[test]
fn add_two_blocks_from_400000() {
    testing_env!(get_context());

    // Check on 400000 block from this answer: https://ethereum.stackexchange.com/a/67333/3032
    let block_height = 400_000;
    let (blocks, hashes) = get_blocks(&WEB3RS, block_height - 1, block_height + 2);

    // $ ../ethrelay/ethashproof/cmd/relayer/relayer 400001
    // digest: 0x3fbea7af642a4e20cd93a945a1f5e23bd72fc5261153e09102cf718980aeff38
    // ethash result: 0x00000000000ca599ebe9913fa00da78a4d1dd2fa154c4fd2aad10ccbca52a2a1
    // Proof length: 24
    // [400001.json]

    let blocks_with_proofs: Vec<BlockWithProofs> = [
        format!("./src/data/{}.json", block_height),
        format!("./src/data/{}.json", block_height + 1),
    ]
    .iter()
    .map(|filename| read_block((&filename).to_string()))
    .collect();

    let mut contract = EthClient::init(
        true,
        400_000 / 30000,
        vec![blocks_with_proofs.first().unwrap().merkle_root],
        blocks[0].clone(),
        30,
        10,
        10,
        None,
    );

    for (block, proof) in blocks
        .into_iter()
        .skip(1)
        .zip(blocks_with_proofs.into_iter())
    {
        contract.add_block_header(block, proof.to_double_node_with_merkle_proof_vec());
    }

    let heights = [block_height as u64, block_height as u64 + 1];
    // Skip parent header hash
    let hashes = &hashes[1..];
    assert_hashes_equal_to_contract_hashes(&contract, &heights, &hashes);
}

#[test]
fn add_blocks_from_12965000() {
    testing_env!(get_context());

    let block_height = 12_965_000;

    // Generated by: node eth2near/eth2near-block-relay/generate.js <block_number>
    let blocks_with_proofs: Vec<BlockWithProofs> = [
        format!("./src/data/{}.json", block_height),
        format!("./src/data/{}.json", block_height + 1),
        format!("./src/data/{}.json", block_height + 2),
        format!("./src/data/{}.json", block_height + 3),
    ]
    .iter()
    .map(|filename| read_block((&filename).to_string()))
    .collect();

    let mut contract = EthClient::init(
        true,
        block_height / 30000,
        vec![blocks_with_proofs.first().unwrap().merkle_root],
        blocks_with_proofs.first().unwrap().header_rlp.0.clone(),
        30,
        10,
        10,
        None,
    );

    for proof in blocks_with_proofs.into_iter().skip(1) {
        contract.add_block_header(
            proof.header_rlp.0.clone(),
            proof.to_double_node_with_merkle_proof_vec(),
        );
    }

    let (_blocks, hashes) = get_blocks(&WEB3RS, block_height as usize, (block_height + 3) as usize);
    let heights = [block_height as u64, block_height as u64 + 1];
    assert_hashes_equal_to_contract_hashes(&contract, &heights, &hashes);
}

#[test]
#[should_panic]
fn add_blocks_with_invalid_mix_hash() {
    testing_env!(get_context());

    let block_height = 12_965_000;

    // Generated by: node eth2near/eth2near-block-relay/generate.js <block_number>
    let blocks_with_proofs: Vec<BlockWithProofs> = [
        format!("./src/data/{}.json", block_height),
        format!("./src/data/{}.json", block_height + 1),
        format!("./src/data/{}.json", block_height + 2),
        format!("./src/data/{}.json", block_height + 3),
    ]
    .iter()
    .map(|filename| read_block((&filename).to_string()))
    .collect();

    let mut contract = EthClient::init(
        true,
        block_height / 30000,
        vec![blocks_with_proofs.first().unwrap().merkle_root],
        blocks_with_proofs.first().unwrap().header_rlp.0.clone(),
        30,
        10,
        10,
        None,
    );

    let proof = &blocks_with_proofs[1];
    let mut header: BlockHeader = rlp::decode(proof.header_rlp.0.clone().as_slice()).unwrap();
    header.mix_hash = H256::from(vec![1; 32]);
    contract.add_block_header(
        rlp::encode(&header).to_vec(),
        proof.to_double_node_with_merkle_proof_vec(),
    );
}

#[test]
#[should_panic(expected = "RlpInconsistentLengthAndData")]
fn add_blocks_with_extra_bytes() {
    testing_env!(get_context());
    let block_height: u64 = 12_965_000;

    // Generated by: node eth2near/eth2near-block-relay/generate.js <block_number>
    let blocks_with_proofs: Vec<BlockWithProofs> = [
        format!("./src/data/{}.json", block_height),
        format!("./src/data/{}.json", block_height + 1),
    ]
    .iter()
    .map(|filename| read_block((&filename).to_string()))
    .collect();

    let mut contract = EthClient::init(
        true,
        block_height / 30000,
        vec![blocks_with_proofs.first().unwrap().merkle_root],
        blocks_with_proofs.first().unwrap().header_rlp.0.clone(),
        30,
        10,
        10,
        None,
    );

    let proof = &blocks_with_proofs[1];
    // Check header rlp with extra byte
    let mut new_rlp = proof.header_rlp.0.clone();
    new_rlp.push(180);

    contract.add_block_header(
        new_rlp.clone(),
        proof.to_double_node_with_merkle_proof_vec(),
    );
}

#[cfg(feature = "expensive_tests")]
#[test]
fn predumped_block_can_be_added() {
    use indicatif::{ProgressBar, ProgressStyle};
    use near_sdk::VMConfig;
    use std::env;
    use std::fs;

    let mut vm_config = VMConfig::free();
    vm_config.limit_config.max_number_logs = u64::MAX;
    vm_config.limit_config.max_total_log_length = u64::MAX;
    testing_env!(get_context(), vm_config, Default::default());

    let mut blocks_with_proofs = fs::read_dir(env::var("ETH_HEADER_DIR").unwrap())
        .unwrap()
        .map(|path| {
            let path = path.unwrap().path();
            (
                path.file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
                path.display().to_string(),
            )
        })
        .collect::<Vec<_>>();
    blocks_with_proofs.sort_by_key(|s| s.0);
    let start_block_height = blocks_with_proofs.first().unwrap().0;

    let first_block_with_proof = read_block(blocks_with_proofs.first().unwrap().1.to_string());

    let mut contract = EthClient::init(
        true,
        start_block_height / 30000,
        vec![first_block_with_proof.merkle_root],
        first_block_with_proof.header_rlp.0.clone(),
        30,
        10,
        10,
        None,
    );

    let bar = ProgressBar::new(blocks_with_proofs.len() as _);
    bar.set_style(ProgressStyle::default_bar().template(
        "[elapsed {elapsed_precise} remaining {eta_precise}] Verifying {bar} {pos:>7}/{len:>7}",
    ));

    for filename in blocks_with_proofs.iter().skip(1) {
        let block_with_proof = read_block(filename.1.to_string());
        contract.add_block_header(
            block_with_proof.header_rlp.0.clone(),
            block_with_proof.to_double_node_with_merkle_proof_vec(),
        );
        assert!(contract.canonical_header_hashes.len() <= 30);
        assert!(contract.all_header_hashes.len() <= 10);
        assert!(contract.headers.len() <= 10);
        assert!(contract.infos.len() <= 10);
        bar.inc(1);
    }
    bar.finish();
}
