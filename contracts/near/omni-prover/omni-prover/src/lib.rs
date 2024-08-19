use near_plugins::{
    access_control, access_control_any, AccessControlRole, AccessControllable, Pausable,
    Upgradable,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, near_bindgen, PanicOnDefault};

#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    PauseManager,
    UpgradableCodeStager,
    UpgradableCodeDeployer,
    DAO,
    BridgesManager,
}

#[derive(BorshSerialize, near_sdk::BorshStorageKey)]
enum StorageKey {
    RegisteredBridges,
}

type ChainKind = String;

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
pub struct OmniProver {
    bridges: near_sdk::collections::UnorderedMap<ChainKind, AccountId>,
}

#[near_bindgen]
impl OmniProver {
    #[init]
    #[private]
    #[must_use]
    pub fn init() -> Self {
        let mut contract = Self {
            bridges: near_sdk::collections::UnorderedMap::new(StorageKey::RegisteredBridges)
        };

        contract.acl_init_super_admin(near_sdk::env::predecessor_account_id());
        contract
    }

    #[access_control_any(roles(Role::BridgesManager, Role::DAO))]
    pub fn set_bridge(&mut self, chain_kind: ChainKind, bridge_account_id: AccountId) {
        self.bridges.insert(&chain_kind, &bridge_account_id);
    }

    #[access_control_any(roles(Role::BridgesManager, Role::DAO))]
    pub fn remove_bridge(&mut self, chain_kind: ChainKind) {
        self.bridges.remove(&chain_kind);
    }

    pub fn get_bridges_list(&self) -> Vec<(ChainKind, AccountId)> {
        self.bridges.iter().collect::<Vec<_>>()
    }

    
}
