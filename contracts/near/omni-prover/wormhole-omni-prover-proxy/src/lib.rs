use near_plugins::{
    access_control, AccessControlRole, AccessControllable, Pausable,
    Upgradable, pause
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Gas, env, ext_contract, near_bindgen, PanicOnDefault, Promise, PromiseError};

/// Gas to call verify_log_entry on prover.
pub const VERIFY_LOG_ENTRY_GAS: Gas = Gas(Gas::ONE_TERA.0 * 50);

#[ext_contract(ext_prover)]
pub trait Prover {
    fn verify_vaa(
        &self,
        vaa: String
    ) -> u32;
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
pub struct WormholeOmniProverProxy {
    pub prover_account: AccountId,
}

#[near_bindgen]
impl WormholeOmniProverProxy {
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
        let vaa = String::from_utf8(msg).unwrap_or_else(|_| env::panic_str("ErrorOnVaaParsing"));

        ext_prover::ext(self.prover_account.clone())
            .with_static_gas(VERIFY_LOG_ENTRY_GAS)
            .verify_vaa(vaa)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(VERIFY_LOG_ENTRY_GAS)
                    .verify_vaa_callback()
            )
    }

    #[private]
    pub fn verify_vaa_callback(
        &mut self,
        #[callback_result] gov_idx: Result<u32, PromiseError>,
    ) -> bool {
        if gov_idx.is_err() {
            return false;
        }

        return true;
    }
}
