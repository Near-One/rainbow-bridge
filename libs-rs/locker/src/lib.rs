/**
* Locker contract for fungible tokens.
*/
use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, ext_contract, Promise};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Locker {
    prover_account: AccountId,
    skip_client_call: bool,
}

impl Default for Locker {
    fn default() -> Self {
        panic!("Fun token should be initialized before usage")
    }
}

#[ext_contract(fun_token)]
pub trait FunToken {
    fn transfer(&mut self, new_owner_id: AccountId, amount: U128);
}

#[ext_contract(prover)]
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

#[ext_contract(ext_locker)]
pub trait ExtLocker {
    #[result_serializer(borsh)]
    fn finish_transfer(&self,
                       #[callback]
                       #[serializer(borsh)] verification_success: bool,
                       #[serializer(borsh)] token_account: AccountId,
                       #[serializer(borsh)] new_owner_id: AccountId,
                       #[serializer(borsh)] amount: U128) -> Promise;
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Proof {
    log_index: u64,
    log_entry_data: Vec<u8>,
    receipt_index: u64,
    receipt_data: Vec<u8>,
    header_data: Vec<u8>,
    proof: Vec<Vec<u8>>,
}

#[near_bindgen]
impl Locker {
    #[init]
    pub fn new(
        #[serializer(borsh)] prover_account: AccountId,
        #[serializer(borsh)] skip_client_call: bool) -> Self {
        Self {
            prover_account,
            skip_client_call
        }
    }

    pub fn unlock_token(&self,
                        #[serializer(borsh)] token_account: AccountId,
                        #[serializer(borsh)] new_owner_id: AccountId,
                        #[serializer(borsh)] amount: U128,
                        #[serializer(borsh)] proof: Proof) -> Promise {
        let Proof {
            log_index,
            log_entry_data,
            receipt_index,
            receipt_data,
            header_data,
            proof,
        } = proof;
        prover::verify_log_entry(
            log_index, log_entry_data, receipt_index, receipt_data, header_data, proof,
            self.skip_client_call,
            &self.prover_account,
            0,
            env::prepaid_gas()/3
        ).then(
            ext_locker::finish_transfer(
                token_account,
                new_owner_id,
                amount,
                &env::current_account_id(),
                0,
                env::prepaid_gas()/3
            )
        )
    }

    pub fn finish_transfer(&self,
                       #[callback] #[serializer(borsh)] verification_success: bool,
                       #[serializer(borsh)] token_account: AccountId,
                       #[serializer(borsh)] new_owner_id: AccountId,
                       #[serializer(borsh)] amount: U128) -> Promise {
        assert_eq!(env::predecessor_account_id(), env::current_account_id(),
                   "Finish transfer is only allowed to be called by the contract itself");
        assert!(verification_success, "Failed to verify the proof");
        fun_token::transfer(new_owner_id, amount, &token_account, 0, env::prepaid_gas()/2)
    }
}

