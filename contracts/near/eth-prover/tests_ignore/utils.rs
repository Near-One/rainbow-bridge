#![allow(dead_code)]
use eth_types::*;
use hex::FromHex;
use near_primitives::transaction::{ExecutionOutcome, ExecutionStatus};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
pub use near_sdk::VMContext;
use serde::{Deserialize, Deserializer};
use workspaces::{network::Sandbox, AccountId, Contract, Worker};

type TxResult = Result<ExecutionOutcome, ExecutionOutcome>;

#[derive(Debug)]
pub struct Hex(pub Vec<u8>);

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

#[derive(BorshSerialize)]
struct EthClientInitArgs {
    validate_ethash: bool,
    dags_start_epoch: u64,
    dags_merkle_roots: Vec<H128>,
    first_header: Vec<u8>,
    hashes_gc_threshold: u64,
    finalized_gc_threshold: u64,
    num_confirmations: u64,
    trusted_signer: Option<AccountId>,
}

#[derive(BorshSerialize)]
struct EthProverInitArgs {
    bridge_smart_contract: AccountId,
}

fn outcome_into_result(outcome: ExecutionOutcome) -> TxResult {
    match outcome.status {
        ExecutionStatus::SuccessValue(_) => Ok(outcome),
        ExecutionStatus::Failure(_) => Err(outcome),
        ExecutionStatus::SuccessReceiptId(_) => panic!("Unresolved ExecutionOutcome run runitme.resolve(tx) to resolve the filnal outcome of tx"),
        ExecutionStatus::Unknown => unreachable!()
    }
}

lazy_static::lazy_static! {
    static ref ETH_PROVER_WASM_BYTES: &'static [u8] = include_bytes!("../../res/eth_prover.wasm").as_ref();
    static ref ETH_CLIENT_WASM_BYTES: &'static [u8] = include_bytes!("../../res/eth_client.wasm").as_ref();
}

pub async fn init_eth_client(worker: &Worker<Sandbox>, validate_ethash: bool) -> Contract {
    let block = read_block("../eth-client/src/data/12965000.json".to_string());
    let init_args = EthClientInitArgs {
        validate_ethash,
        dags_start_epoch: 0,
        dags_merkle_roots: read_roots_collection().dag_merkle_roots,
        first_header: block.header(),
        hashes_gc_threshold: 400000,
        finalized_gc_threshold: 500,
        num_confirmations: 5,
        trusted_signer: None,
    };

    let contract = worker.dev_deploy(&ETH_CLIENT_WASM_BYTES).await.unwrap();

    let _result = worker
        .root_account()
        .unwrap()
        .transfer_near(&contract.id(), 30)
        .await
        .unwrap();

    let _result = contract
        .call("init")
        .args(init_args.try_to_vec().unwrap())
        .max_gas()
        .transact()
        .await
        .unwrap()
        .unwrap();

    contract
}

pub async fn init_eth_prover(
    worker: &Worker<Sandbox>,
    eth_client_account_id: AccountId,
) -> Contract {
    let init_args = EthProverInitArgs {
        bridge_smart_contract: eth_client_account_id,
    };

    let contract = worker.dev_deploy(&ETH_PROVER_WASM_BYTES).await.unwrap();

    let _result = worker
        .root_account()
        .unwrap()
        .transfer_near(&contract.id(), 30)
        .await
        .unwrap();

    let _result = contract
        .call("init")
        .args(init_args.try_to_vec().unwrap())
        .max_gas()
        .transact()
        .await
        .unwrap()
        .unwrap();

    contract
}

fn read_roots_collection() -> RootsCollection {
    read_roots_collection_raw().into()
}

fn read_roots_collection_raw() -> RootsCollectionRaw {
    serde_json::from_reader(
        std::fs::File::open(std::path::Path::new(
            "../eth-client/src/data/dag_merkle_roots.json",
        ))
        .unwrap(),
    )
    .unwrap()
}

#[derive(Debug, Deserialize)]
struct RootsCollectionRaw {
    pub dag_merkle_roots: Vec<Hex>, // H128
}

#[derive(Debug, Deserialize)]
pub struct RootsCollection {
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

pub fn read_block(filename: String) -> BlockWithProofs {
    read_block_raw(filename).into()
}

fn read_block_raw(filename: String) -> BlockWithProofsRaw {
    serde_json::from_reader(std::fs::File::open(std::path::Path::new(&filename)).unwrap()).unwrap()
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
pub struct BlockWithProofs {
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

#[derive(Default, Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct DoubleNodeWithMerkleProof {
    pub dag_nodes: Vec<H512>, // [H512; 2]
    pub proof: Vec<H128>,
}

#[derive(BorshSerialize)]
pub struct AddBlockHeaderArgs {
    pub block_header: Vec<u8>,
    pub dag_nodes: Vec<DoubleNodeWithMerkleProof>,
}

impl BlockWithProofs {
    pub fn header(&self) -> Vec<u8> {
        self.header_rlp.0.clone()
    }

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

#[derive(BorshSerialize)]
pub struct AssertEthbridgeHashArgs {
    pub block_number: u64,
    pub expected_block_hash: H256,
}

pub fn get_context(input: Vec<u8>) -> VMContext {
    VMContext {
        current_account_id: "alice.near".parse().unwrap(),
        signer_account_id: "bob.near".parse().unwrap(),
        signer_account_pk: vec![0u8; 33].try_into().unwrap(),
        predecessor_account_id: "carol.near".parse().unwrap(),
        input,
        block_index: 0,
        block_timestamp: 0,
        account_balance: 0,
        account_locked_balance: 0,
        epoch_height: 0,
        storage_usage: 0,
        attached_deposit: 0,
        prepaid_gas: near_sdk::Gas(1_000_000),
        random_seed: vec![1; 32].try_into().unwrap(),
        view_config: None,
        output_data_receivers: vec![],
    }
}
