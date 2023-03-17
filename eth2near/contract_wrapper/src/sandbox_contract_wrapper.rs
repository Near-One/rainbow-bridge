use crate::contract_wrapper_trait::ContractWrapper;
use near_crypto::KeyType::ED25519;
use near_crypto::PublicKey;
use near_primitives::errors::{ActionError, ActionErrorKind, TxExecutionError};
use near_primitives::types::AccountId;
use near_primitives::views::{
    ExecutionOutcomeView, ExecutionOutcomeWithIdView, ExecutionStatusView,
    FinalExecutionOutcomeView, FinalExecutionStatus, SignedTransactionView,
};
use near_sdk::{Balance, Gas};
use std::error::Error;
use std::future::IntoFuture;
use tokio::runtime::Runtime;
use workspaces::{Account, Contract};

pub const MAX_GAS: Gas = Gas(Gas::ONE_TERA.0 * 300);

/// Implementation for interaction with NEAR contract in Sandbox emulator for testing purposes.
/// Implemented using https://github.com/near/workspaces-rs
pub struct SandboxContractWrapper {
    /// Account which signs transactions
    signer_account: Account,

    /// Emulated NEAR contract
    contract: Contract,
}

impl SandboxContractWrapper {
    /// `SandboxContractWrapper` constructor
    pub fn new(signer_account: &Account, contract: Contract) -> Self {
        SandboxContractWrapper {
            signer_account: signer_account.clone(),
            contract,
        }
    }

    fn get_final_execution_outcome_view_from_call_execution_details(
        call_execution_details: workspaces::result::ExecutionFinalResult,
    ) -> FinalExecutionOutcomeView {
        let status = match call_execution_details.is_success() {
            true => FinalExecutionStatus::SuccessValue("".into()),
            false => FinalExecutionStatus::Failure(TxExecutionError::ActionError(ActionError {
                index: None,
                kind: ActionErrorKind::AccountAlreadyExists {
                    account_id: "fail.testnet".parse().unwrap(),
                },
            })),
        };

        let outcome = call_execution_details.outcome();

        FinalExecutionOutcomeView {
            status: status,
            transaction: SignedTransactionView {
                signer_id: "fake_signature_id".parse().unwrap(),
                public_key: PublicKey::empty(ED25519),
                nonce: 0,
                receiver_id: "fake_receiver_id".parse().unwrap(),
                actions: vec![],
                signature: Default::default(),
                hash: Default::default(),
            },
            transaction_outcome: ExecutionOutcomeWithIdView {
                proof: vec![],
                block_hash: Default::default(),
                id: Default::default(),
                outcome: ExecutionOutcomeView {
                    logs: outcome.clone().logs,
                    receipt_ids: vec![],
                    gas_burnt: outcome.gas_burnt,
                    tokens_burnt: outcome.tokens_burnt,
                    executor_id: outcome.executor_id.parse().unwrap(),
                    status: ExecutionStatusView::Unknown,
                    metadata: Default::default(),
                },
            },
            receipts_outcome: vec![],
        }
    }
}

impl ContractWrapper for SandboxContractWrapper {
    fn get_account_id(&self) -> AccountId {
        self.contract.id().parse().unwrap()
    }

    fn get_signer_account_id(&self) -> AccountId {
        self.signer_account.id().to_string().parse().unwrap()
    }

    fn call_view_function(
        &self,
        method_name: String,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let rt = Runtime::new()?;

        Ok(rt
            .block_on(self.contract.view(&method_name).args(args).into_future())
            .unwrap()
            .result)
    }

    fn call_change_method_batch(
        &self,
        method_name: Vec<String>,
        args: Vec<Vec<u8>>,
        deposit: Option<Vec<Balance>>,
        gas: Option<Gas>,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn Error>> {
        let deposit = match deposit {
            Some(deposit) => Some(deposit[0]),
            None => None::<Balance>,
        };

        for i in 0..method_name.len() - 1 {
            self.call_change_method(method_name[i].clone(), args[i].clone(), deposit, gas)
                .unwrap();
        }

        self.call_change_method(
            method_name[method_name.len() - 1].clone(),
            args[method_name.len() - 1].clone(),
            deposit,
            gas,
        )
    }

    fn call_change_method(
        &self,
        method_name: String,
        args: Vec<u8>,
        deposit: Option<Balance>,
        gas: Option<Gas>,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn Error>> {
        let rt = Runtime::new()?;

        Ok(
            Self::get_final_execution_outcome_view_from_call_execution_details(
                rt.block_on(
                    self.signer_account
                        .call(self.contract.id(), &method_name)
                        .deposit(match deposit {
                            Some(deposit) => deposit,
                            None => 0,
                        })
                        .gas(match gas {
                            Some(gas) => gas.0,
                            None => MAX_GAS.0,
                        })
                        .args(args)
                        .transact(),
                )?,
            ),
        )
    }
}
