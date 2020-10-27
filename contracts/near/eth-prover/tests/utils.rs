#![allow(dead_code)]
use borsh::{BorshDeserialize, BorshSerialize};
use eth_types::*;
use hex::FromHex;
use near_crypto::{InMemorySigner, KeyType, Signer};
use near_primitives::{
    account::{AccessKey, Account},
    errors::{RuntimeError, TxExecutionError},
    hash::CryptoHash,
    transaction::{ExecutionOutcome, ExecutionStatus, Transaction},
    types::{AccountId, Balance},
};
use near_runtime_standalone::init_runtime_and_signer;
pub use near_runtime_standalone::RuntimeStandalone;
pub use near_sdk::VMContext;
use serde::{Deserialize, Deserializer};

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

pub fn ntoy(near_amount: Balance) -> Balance {
    near_amount * 10u128.pow(24)
}

pub struct ExternalUser {
    pub account_id: AccountId,
    pub signer: InMemorySigner,
}

impl ExternalUser {
    pub fn new(account_id: AccountId, signer: InMemorySigner) -> Self {
        Self { account_id, signer }
    }

    #[allow(dead_code)]
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    #[allow(dead_code)]
    pub fn signer(&self) -> &InMemorySigner {
        &self.signer
    }

    pub fn account(&self, runtime: &RuntimeStandalone) -> Account {
        runtime
            .view_account(&self.account_id)
            .expect("Account should be there")
    }

    pub fn create_external(
        &self,
        runtime: &mut RuntimeStandalone,
        new_account_id: AccountId,
        amount: Balance,
    ) -> Result<ExternalUser, ExecutionOutcome> {
        let new_signer =
            InMemorySigner::from_seed(&new_account_id, KeyType::ED25519, &new_account_id);
        let tx = self
            .new_tx(runtime, new_account_id.clone())
            .create_account()
            .add_key(new_signer.public_key(), AccessKey::full_access())
            .transfer(amount)
            .sign(&self.signer);
        let res = runtime.resolve_tx(tx);

        // TODO: this temporary hack, must be rewritten
        if let Err(err) = res.clone() {
            if let RuntimeError::InvalidTxError(tx_err) = err {
                let mut out = ExecutionOutcome::default();
                out.status = ExecutionStatus::Failure(TxExecutionError::InvalidTxError(tx_err));
                return Err(out);
            } else {
                unreachable!();
            }
        } else {
            outcome_into_result(res.unwrap())?;
            runtime.process_all().unwrap();
            Ok(ExternalUser {
                account_id: new_account_id,
                signer: new_signer,
            })
        }
    }

    pub fn transfer(
        &self,
        runtime: &mut RuntimeStandalone,
        receiver_id: &str,
        amount: Balance,
    ) -> TxResult {
        let tx = self
            .new_tx(runtime, receiver_id.to_string())
            .transfer(amount)
            .sign(&self.signer);
        let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        outcome_into_result(res)
    }

    pub fn function_call(
        &self,
        runtime: &mut RuntimeStandalone,
        receiver_id: &str,
        method: &str,
        args: &[u8],
        deposit: u128,
    ) -> TxResult {
        let tx = self
            .new_tx(runtime, receiver_id.to_string())
            .function_call(method.into(), args.to_vec(), 300000000000000, deposit)
            .sign(&self.signer);
        let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        outcome_into_result(res)
    }

    pub fn init_eth_client(
        &self,
        runtime: &mut RuntimeStandalone,
        eth_client_account_id: AccountId,
        validate_ethash: bool,
    ) -> TxResult {
        let block = read_block("../eth-client/src/data/10234001.json".to_string());
        let init_args = EthClientInitArgs {
            validate_ethash,
            dags_start_epoch: 0,
            dags_merkle_roots: read_roots_collection().dag_merkle_roots,
            first_header: block.header(),
            hashes_gc_threshold: 400000,
            finalized_gc_threshold: 500,
            num_confirmations: 10,
            trusted_signer: None,
        };
        let tx = self
            .new_tx(runtime, eth_client_account_id)
            .create_account()
            .transfer(ntoy(30))
            .deploy_contract(ETH_CLIENT_WASM_BYTES.to_vec())
            .function_call(
                "init".into(),
                init_args.try_to_vec().unwrap(),
                1000000000000000,
                0,
            )
            .sign(&self.signer);
        let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        outcome_into_result(res)
    }

    pub fn init_eth_prover(
        &self,
        runtime: &mut RuntimeStandalone,
        eth_prover_account_id: AccountId,
        eth_client_account_id: AccountId,
    ) -> TxResult {
        let init_args = EthProverInitArgs {
            bridge_smart_contract: eth_client_account_id,
        };
        let tx = self
            .new_tx(runtime, eth_prover_account_id)
            .create_account()
            .transfer(ntoy(30))
            .deploy_contract(ETH_PROVER_WASM_BYTES.to_vec())
            .function_call(
                "init".into(),
                init_args.try_to_vec().unwrap(),
                1000000000000000,
                0,
            )
            .sign(&self.signer);
        let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        outcome_into_result(res)
    }

    fn new_tx(&self, runtime: &RuntimeStandalone, receiver_id: AccountId) -> Transaction {
        let nonce = runtime
            .view_access_key(&self.account_id, &self.signer.public_key())
            .unwrap()
            .nonce
            + 1;
        Transaction::new(
            self.account_id.clone(),
            self.signer.public_key(),
            receiver_id,
            nonce,
            CryptoHash::default(),
        )
    }
}

pub fn new_root(account_id: AccountId) -> (RuntimeStandalone, ExternalUser) {
    let (runtime, signer) = init_runtime_and_signer(&account_id);
    (runtime, ExternalUser { account_id, signer })
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

pub fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
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
        epoch_height: 0,
        storage_usage: 0,
        attached_deposit: 0,
        prepaid_gas: 10u64.pow(18),
        random_seed: vec![0, 1, 2],
        is_view,
        output_data_receivers: vec![],
    }
}
