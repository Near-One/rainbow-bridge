#![allow(dead_code)]
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Deserializer};
use serde_json::{json};
use near_crypto::{InMemorySigner, KeyType, Signer};
use eth_types::*;
use near_primitives::{
    account::{AccessKey, Account},
    errors::{RuntimeError, TxExecutionError},
    hash::CryptoHash,
    transaction::{ExecutionOutcome, ExecutionStatus, Transaction},
    types::{AccountId, Balance},
};
use near_runtime_standalone::{init_runtime_and_signer};
pub use near_runtime_standalone::RuntimeStandalone;

type TxResult = Result<ExecutionOutcome, ExecutionOutcome>;

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


    pub fn init_eth_client(
        &self,
        runtime: &mut RuntimeStandalone,
        eth_client_account_id: AccountId,
        validate_ethash: bool,
    ) -> TxResult {
        println!("{:?}", &json!({
            "validate_ethash": validate_ethash,
            "dags_start_epoch": 0,
            "dags_merkle_roots": read_roots_collection_raw().dag_merkle_roots,
        }));
        let tx = self
            .new_tx(runtime, eth_client_account_id)
            .create_account()
            .transfer(ntoy(30))
            .deploy_contract(ETH_CLIENT_WASM_BYTES.to_vec())
            .function_call(
                "init".into(),
                serde_json::to_vec(&json!({
                    "validate_ethash": validate_ethash,
                    "dags_start_epoch": 0,
                    "dags_merkle_roots": read_roots_collection_raw().dag_merkle_roots,
                })).unwrap(),
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
        eth_client_account_id: &str,
    ) -> TxResult {
        let tx = self
            .new_tx(runtime, eth_prover_account_id)
            .create_account()
            .transfer(ntoy(30))
            .deploy_contract(ETH_PROVER_WASM_BYTES.to_vec())
            .function_call(
                "init".into(),
                serde_json::to_vec(&json!({
                    "bridge_smart_contract": eth_client_account_id
                })).unwrap(),
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

#[derive(Debug, Deserialize)]
struct RootsCollectionRaw {
    pub dag_merkle_roots: Vec<String>,
}

fn read_roots_collection_raw() -> RootsCollectionRaw {
    serde_json::from_reader(
        std::fs::File::open(std::path::Path::new("../eth-client/src/data/dag_merkle_roots.json")).unwrap(),
    )
    .unwrap()
}