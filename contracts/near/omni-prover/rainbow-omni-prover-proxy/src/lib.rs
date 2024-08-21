use hex::ToHex;
use bridge_common::prover::{EthAddress, EthEvent, EthEventParams};
use ethabi::ParamType;
use near_plugins::{
    access_control, AccessControlRole, AccessControllable, Pausable,
    Upgradable, pause
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Gas, env, ext_contract, near_bindgen, PanicOnDefault, Promise, Balance};
use omni_types::{OmniAddress, BridgeMessage};

/// Gas to call verify_log_entry on prover.
pub const VERIFY_LOG_ENTRY_GAS: Gas = Gas(Gas::ONE_TERA.0 * 50);

#[ext_contract(ext_prover)]
pub trait Prover {
    #[result_serializer(borsh)]
    fn verify_log_entry(
        &self,
        #[serializer(borsh)] log_index: u64,
        #[serializer(borsh)] log_entry_data: Vec<u8>,
        #[serializer(borsh)] receipt_index: u64,
        #[serializer(borsh)] receipt_data: Vec<u8>,
        #[serializer(borsh)] header_data: Vec<u8>,
        #[serializer(borsh)] proof: Vec<Vec<u8>>,
        #[serializer(borsh)] skip_bridge_call: bool,
    ) -> bool;
}

#[derive(Default, BorshDeserialize, BorshSerialize, Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Proof {
    pub log_index: u64,
    pub log_entry_data: Vec<u8>,
    pub receipt_index: u64,
    pub receipt_data: Vec<u8>,
    pub header_data: Vec<u8>,
    pub proof: Vec<Vec<u8>>,
}

/// Data that was emitted by the Ethereum Unlocked event.
#[derive(Debug, Eq, PartialEq)]
pub struct EthUnlockedEvent {
    pub eth_factory_address: EthAddress,
    pub token: String,
    pub sender: String,
    pub amount: Balance,
    pub recipient: String,
    pub token_eth_address: EthAddress,
}

impl EthUnlockedEvent {
    fn event_params() -> EthEventParams {
        vec![
            ("token".to_string(), ParamType::String, false),
            ("sender".to_string(), ParamType::Address, true),
            ("amount".to_string(), ParamType::Uint(256), false),
            ("recipient".to_string(), ParamType::String, false),
            ("tokenEthAddress".to_string(), ParamType::Address, true),
        ]
    }

    /// Parse raw log entry data.
    pub fn from_log_entry_data(data: &[u8]) -> Self {
        let event =
            EthEvent::from_log_entry_data("Withdraw", EthUnlockedEvent::event_params(), data);
        let token = event.log.params[0].value.clone().to_string().unwrap();
        let sender = event.log.params[1].value.clone().to_address().unwrap().0;
        let sender = (&sender).encode_hex::<String>();
        let amount = event.log.params[2]
            .value
            .clone()
            .to_uint()
            .unwrap()
            .as_u128();
        let recipient = event.log.params[3].value.clone().to_string().unwrap();
        let token_eth_address = event.log.params[4].value.clone().to_address().unwrap().0;
        Self {
            eth_factory_address: event.locker_address,
            token,
            sender,
            amount,
            recipient,
            token_eth_address,
        }
    }
}

#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    PauseManager,
    UpgradableCodeStager,
    UpgradableCodeDeployer,
    DAO,
    UnrestrictedValidateProof,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Pausable, Upgradable)]
#[access_control(role_type(Role))]
#[pausable(manager_roles(Role::PauseManager, Role::DAO))]
#[upgradable(access_control_roles(
    code_stagers(Role::UpgradableCodeStager, Role::DAO),
    code_deployers(Role::UpgradableCodeDeployer, Role::DAO),
    duration_initializers(Role::DAO),
    duration_update_stagers(Role::DAO),
    duration_update_appliers(Role::DAO),
))]
pub struct RainbowOmniProverProxy {
    pub prover_account: AccountId,
}

#[near_bindgen]
impl RainbowOmniProverProxy {
    #[init]
    #[private]
    #[must_use]
    pub fn init(prover_account: AccountId) -> Self {
        let mut contract = Self {
            prover_account
        };

        contract.acl_init_super_admin(near_sdk::env::predecessor_account_id());
        contract
    }

    #[pause(except(roles(Role::UnrestrictedValidateProof, Role::DAO)))]
    pub fn verify_proof(
        &self,
        msg: Vec<u8>,
    ) -> Promise {
        let proof = Proof::try_from_slice(&msg).unwrap_or_else(|_| env::panic_str("ErrorOnProofParsing"));

        ext_prover::ext(self.prover_account.clone())
            .with_static_gas(VERIFY_LOG_ENTRY_GAS)
            .verify_log_entry(
                proof.log_index,
                proof.log_entry_data.clone(),
                proof.receipt_index,
                proof.receipt_data,
                proof.header_data,
                proof.proof,
                false, // Do not skip bridge call. This is only used for development and diagnostics.
            ).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(VERIFY_LOG_ENTRY_GAS)
                    .verify_log_entry_callback(proof.log_entry_data)
            )
    }

    #[private]
    pub fn verify_log_entry_callback(
        &mut self,
        log_entry_data: Vec<u8>,
        #[callback_result] is_valid: bool,
    ) -> Option<BridgeMessage> {
        if !is_valid {
            return None;
        }

        let event = EthUnlockedEvent::from_log_entry_data(&log_entry_data);
        return Some(BridgeMessage{
            token_id: OmniAddress{chain: "near".to_string(), account: event.token},
            sender: OmniAddress{chain: "eth".to_string(), account: event.sender},
            receiver: OmniAddress{chain: "near".to_string(), account: event.recipient},
            amount: event.amount
        });
    }
}
