/**
* Mintable Fungible Token implementation with JSON serialization.
* NOTES:

Properties specific to Mintable Fungible Token:
*  - It is an extension of the standard Fungible token that can be found here: https://github.com/near/near-sdk-rs/tree/master/examples/fungible-token
*  - The total balance is not fixed, and is initially 0. When valid proof is submitted the total
*    balance increases, when the tokens are burnt the balance decreases.
*  - The contract permanently memorizes the hashes of the events that were used for
     minting tokens;

Properties inherited from the standard Fungible Token:
*  - The maximum balance value is limited by U128 (2**128 - 1).
*  - JSON calls should pass U128 as a base-10 string. E.g. "100".
*  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
*    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
*  - The contract tracks the change in storage before and after the call. If the storage increases,
*    the contract requires the caller of the contract to attach enough deposit to the function call
*    to cover the storage cost.
*    This is done to prevent a denial of service attack on the contract by taking all available storage.
*    If the storage decreases, the contract will issue a refund for the cost of the released storage.
*    The unused tokens from the attached deposit are also refunded, so it's safe to
*    attach more deposit than required.
*  - To prevent the deployed contract from being modified or deleted, it should not have any access
*    keys on its account.
*/
use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, Balance, Promise, PromiseOrValue, StorageUsage,
};
#[cfg(test)]
use serde::Deserialize;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Price per 1 byte of storage from mainnet genesis config.
const STORAGE_PRICE_PER_BYTE: Balance = 100000000000000000000;

/// Contains balance and allowances information for one account.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    /// Current account balance.
    pub balance: Balance,
    /// Escrow Account ID hash to the allowance amount.
    /// Allowance is the amount of tokens the Escrow Account ID can spent on behalf of the account
    /// owner.
    pub allowances: UnorderedMap<Vec<u8>, Balance>,
}

impl Account {
    /// Initializes a new Account with 0 balance and no allowances for a given `account_hash`.
    pub fn new(account_hash: Vec<u8>) -> Self {
        Self {
            balance: 0,
            allowances: UnorderedMap::new(account_hash),
        }
    }

    /// Sets allowance for account `escrow_account_id` to `allowance`.
    pub fn set_allowance(&mut self, escrow_account_id: &AccountId, allowance: Balance) {
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        if allowance > 0 {
            self.allowances.insert(&escrow_hash, &allowance);
        } else {
            self.allowances.remove(&escrow_hash);
        }
    }

    /// Returns the allowance of account `escrow_account_id`.
    pub fn get_allowance(&self, escrow_account_id: &AccountId) -> Balance {
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        self.allowances.get(&escrow_hash).unwrap_or(0)
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MintableFungibleToken {
    /// sha256(AccountID) -> Account details.
    pub accounts: UnorderedMap<Vec<u8>, Account>,

    /// Total supply of the all token.
    pub total_supply: Balance,
    /// The account of the prover that we can use to prove
    pub prover_account: AccountId,
    /// Address of the Ethereum locker contract.
    pub locker_address: [u8; 20],
    /// Hashes of the events that were already used.
    pub used_events: UnorderedSet<Vec<u8>>,
}

impl Default for MintableFungibleToken {
    fn default() -> Self {
        panic!("Fun token should be initialized before usage")
    }
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

#[cfg(not(test))]
#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct Proof {
    log_index: u64,
    log_entry_data: Vec<u8>,
    receipt_index: u64,
    receipt_data: Vec<u8>,
    header_data: Vec<u8>,
    proof: Vec<Vec<u8>>,
}

#[cfg(test)]
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Clone)]
pub struct Proof {
    log_index: u64,
    log_entry_data: Vec<u8>,
    receipt_index: u64,
    receipt_data: Vec<u8>,
    header_data: Vec<u8>,
    proof: Vec<Vec<u8>>,
}

/// Data that was emitted by the Ethereum event.
pub struct EthEventData {
    pub locker_address: [u8; 20],
    pub token: String,
    pub sender: String,
    pub amount: Balance,
    pub recipient: AccountId,
}

impl EthEventData {
    /// Parse raw log entry data.
    pub fn from_log_entry_data(data: &[u8]) -> Self {
        use eth_types::*;
        use ethabi::{Event, EventParam, Hash, ParamType, RawLog};
        use hex::ToHex;

        let event = Event {
            name: "Locked".to_string(),
            inputs: vec![
                EventParam {
                    name: "token".to_string(),
                    kind: ParamType::Address,
                    indexed: true,
                },
                EventParam {
                    name: "sender".to_string(),
                    kind: ParamType::Address,
                    indexed: true,
                },
                EventParam {
                    name: "amount".to_string(),
                    kind: ParamType::Uint(256),
                    indexed: false,
                },
                EventParam {
                    name: "accountId".to_string(),
                    kind: ParamType::String,
                    indexed: false,
                },
            ],
            anonymous: false,
        };

        let log_entry: LogEntry = rlp::decode(data).unwrap();
        let locker_address = (log_entry.address.clone().0).0;
        let raw_log = RawLog {
            topics: log_entry
                .topics
                .iter()
                .map(|h| Hash::from(&((h.0).0)))
                .collect(),
            data: log_entry.data.clone(),
        };
        let log = event.parse_log(raw_log).unwrap();
        let token = log.params[0].value.clone().to_address().unwrap().0;
        let token = (&token).encode_hex::<String>();
        let sender = log.params[1].value.clone().to_address().unwrap().0;
        let sender = (&sender).encode_hex::<String>();
        let amount = log.params[2].value.clone().to_uint().unwrap().as_u128();
        let recipient = log.params[3].value.clone().to_string().unwrap();
        Self {
            locker_address,
            token,
            sender,
            amount,
            recipient,
        }
    }
}

impl std::fmt::Display for EthEventData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "token: {}; sender: {}; amount: {}; recipient: {}",
            self.token, self.sender, self.amount, self.recipient
        )
    }
}

#[ext_contract(ext_fungible_token)]
pub trait ExtFungibleToken {
    #[result_serializer(borsh)]
    fn finish_mint(
        &self,
        #[callback]
        #[serializer(borsh)]
        verification_success: bool,
        #[serializer(borsh)] new_owner_id: AccountId,
        #[serializer(borsh)] amount: U128,
    ) -> Promise;
}

#[near_bindgen]
impl MintableFungibleToken {
    /// Initializes the contract without total supply.
    /// `prover_account`: NEAR account of the Near Prover contract;
    /// `locker_address`: Ethereum address of the locker contract, in hex.
    #[init]
    pub fn new(prover_account: AccountId, locker_address: String) -> Self {
        let data =
            hex::decode(locker_address).expect("`locker_address` should be a valid hex string.");
        assert_eq!(data.len(), 20, "`locker_address` should be 20 bytes long");
        let mut locker_address = [0u8; 20];
        locker_address.copy_from_slice(&data);
        assert!(!env::state_exists(), "Already initialized");
        Self {
            accounts: UnorderedMap::new(b"a".to_vec()),
            total_supply: 0,
            prover_account,
            locker_address,
            used_events: UnorderedSet::new(b"u".to_vec()),
        }
    }

    /// Increments the `allowance` for `escrow_account_id` by `amount` on the account of the caller of this contract
    /// (`predecessor_id`) who is the balance owner.
    /// Requirements:
    /// * Caller of the method has to attach deposit enough to cover storage difference at the
    ///   fixed storage price defined in the contract.
    #[payable]
    pub fn inc_allowance(&mut self, escrow_account_id: AccountId, amount: U128) {
        let initial_storage = env::storage_usage();
        assert!(
            env::is_valid_account_id(escrow_account_id.as_bytes()),
            "Escrow account ID is invalid"
        );
        let owner_id = env::predecessor_account_id();
        if escrow_account_id == owner_id {
            env::panic(b"Can not increment allowance for yourself");
        }
        let mut account = self.get_account(&owner_id);
        let current_allowance = account.get_allowance(&escrow_account_id);
        account.set_allowance(
            &escrow_account_id,
            current_allowance.saturating_add(amount.0),
        );
        self.set_account(&owner_id, &account);
        self.refund_storage(initial_storage);
    }

    /// Decrements the `allowance` for `escrow_account_id` by `amount` on the account of the caller of this contract
    /// (`predecessor_id`) who is the balance owner.
    /// Requirements:
    /// * Caller of the method has to attach deposit enough to cover storage difference at the
    ///   fixed storage price defined in the contract.
    #[payable]
    pub fn dec_allowance(&mut self, escrow_account_id: AccountId, amount: U128) {
        let initial_storage = env::storage_usage();
        assert!(
            env::is_valid_account_id(escrow_account_id.as_bytes()),
            "Escrow account ID is invalid"
        );
        let owner_id = env::predecessor_account_id();
        if escrow_account_id == owner_id {
            env::panic(b"Can not decrement allowance for yourself");
        }
        let mut account = self.get_account(&owner_id);
        let current_allowance = account.get_allowance(&escrow_account_id);
        account.set_allowance(
            &escrow_account_id,
            current_allowance.saturating_sub(amount.0),
        );
        self.set_account(&owner_id, &account);
        self.refund_storage(initial_storage);
    }

    /// Transfers the `amount` of tokens from `owner_id` to the `new_owner_id`.
    /// Requirements:
    /// * `amount` should be a positive integer.
    /// * `owner_id` should have balance on the account greater or equal than the transfer `amount`.
    /// * If this function is called by an escrow account (`owner_id != predecessor_account_id`),
    ///   then the allowance of the caller of the function (`predecessor_account_id`) on
    ///   the account of `owner_id` should be greater or equal than the transfer `amount`.
    /// * Caller of the method has to attach deposit enough to cover storage difference at the
    ///   fixed storage price defined in the contract.
    #[payable]
    pub fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, amount: U128) {
        let initial_storage = env::storage_usage();
        assert!(
            env::is_valid_account_id(new_owner_id.as_bytes()),
            "New owner's account ID is invalid"
        );
        let amount = amount.into();
        if amount == 0 {
            env::panic(b"Can't transfer 0 tokens");
        }
        assert_ne!(
            owner_id, new_owner_id,
            "The new owner should be different from the current owner"
        );
        // Retrieving the account from the state.
        let mut account = self.get_account(&owner_id);

        // Checking and updating unlocked balance
        if account.balance < amount {
            env::panic(b"Not enough balance");
        }
        account.balance -= amount;

        // If transferring by escrow, need to check and update allowance.
        let escrow_account_id = env::predecessor_account_id();
        if escrow_account_id != owner_id {
            let allowance = account.get_allowance(&escrow_account_id);
            if allowance < amount {
                env::panic(b"Not enough allowance");
            }
            account.set_allowance(&escrow_account_id, allowance - amount);
        }

        // Saving the account back to the state.
        self.set_account(&owner_id, &account);

        // Deposit amount to the new owner and save the new account to the state.
        let mut new_account = self.get_account(&new_owner_id);
        new_account.balance += amount;
        self.set_account(&new_owner_id, &new_account);
        self.refund_storage(initial_storage);
    }

    /// Transfer `amount` of tokens from the caller of the contract (`predecessor_id`) to
    /// `new_owner_id`.
    /// Act the same was as `transfer_from` with `owner_id` equal to the caller of the contract
    /// (`predecessor_id`).
    /// Requirements:
    /// * Caller of the method has to attach deposit enough to cover storage difference at the
    ///   fixed storage price defined in the contract.
    #[payable]
    pub fn transfer(&mut self, new_owner_id: AccountId, amount: U128) {
        // NOTE: New owner's Account ID checked in transfer_from.
        // Storage fees are also refunded in transfer_from.
        self.transfer_from(env::predecessor_account_id(), new_owner_id, amount);
    }

    /// Returns total supply of tokens.
    pub fn get_total_supply(&self) -> U128 {
        self.total_supply.into()
    }

    /// Returns balance of the `owner_id` account.
    pub fn get_balance(&self, owner_id: AccountId) -> U128 {
        self.get_account(&owner_id).balance.into()
    }

    /// Returns current allowance of `escrow_account_id` for the account of `owner_id`.
    ///
    /// NOTE: Other contracts should not rely on this information, because by the moment a contract
    /// receives this information, the allowance may already be changed by the owner.
    /// So this method should only be used on the front-end to see the current allowance.
    pub fn get_allowance(&self, owner_id: AccountId, escrow_account_id: AccountId) -> U128 {
        assert!(
            env::is_valid_account_id(escrow_account_id.as_bytes()),
            "Escrow account ID is invalid"
        );
        self.get_account(&owner_id)
            .get_allowance(&escrow_account_id)
            .into()
    }

    /// Mint the token, increasing the total supply given the proof that the mirror token was locked
    /// on the Ethereum blockchain.
    #[payable]
    pub fn mint(&mut self, #[serializer(borsh)] proof: Proof) -> PromiseOrValue<()> {
        let initial_storage = env::storage_usage();
        self.record_proof(&proof);
        let current_storage = env::storage_usage();
        let attached_deposit = env::attached_deposit();
        let required_deposit =
            Balance::from(current_storage - initial_storage) * STORAGE_PRICE_PER_BYTE;
        let leftover_deposit = attached_deposit - required_deposit;
        let Proof {
            log_index,
            log_entry_data,
            receipt_index,
            receipt_data,
            header_data,
            proof,
        } = proof;
        let event = EthEventData::from_log_entry_data(&log_entry_data);
        assert_eq!(
            event.locker_address,
            self.locker_address,
            "Event's address {} does not match locker address of this token {}",
            hex::encode(&event.locker_address),
            hex::encode(&self.locker_address),
        );
        env::log(format!("{}", event).as_bytes());
        let EthEventData {
            recipient, amount, ..
        } = event;
        if cfg!(test) {
            // In unit test we don't do cross contract call into prover contract
            self.finish_mint(true, recipient, amount.into());
            PromiseOrValue::Value(())
        } else {
            PromiseOrValue::Promise(
                prover::verify_log_entry(
                    log_index,
                    log_entry_data,
                    receipt_index,
                    receipt_data,
                    header_data,
                    proof,
                    false, // Do not skip bridge call. This is only used for development and diagnostics.
                    &self.prover_account,
                    0,
                    env::prepaid_gas() / 3,
                )
                .then(ext_fungible_token::finish_mint(
                    recipient,
                    amount.into(),
                    &env::current_account_id(),
                    leftover_deposit,
                    env::prepaid_gas() / 3,
                )),
            )
        }
    }

    /// Finish minting once the proof was successfully validated. Can only be called by the contract
    /// itself.
    #[payable]
    pub fn finish_mint(
        &mut self,
        #[callback]
        #[serializer(borsh)]
        verification_success: bool,
        #[serializer(borsh)] new_owner_id: AccountId,
        #[serializer(borsh)] amount: U128,
    ) {
        let initial_storage = env::storage_usage();
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Finish transfer is only allowed to be called by the contract itself"
        );
        assert!(verification_success, "Failed to verify the proof");

        let mut account = self.get_account(&new_owner_id);
        let amount: Balance = amount.into();
        account.balance += amount;
        self.total_supply += amount;
        self.set_account(&new_owner_id, &account);
        self.refund_storage(initial_storage);
    }

    /// Burn given amount of tokens and unlock it on the Ethereum side for the recipient address.
    /// We return the amount as u128 and the address of the beneficiary as `[u8; 20]` for ease of
    /// processing on Solidity side.
    #[result_serializer(borsh)]
    pub fn burn(&mut self, amount: U128, recipient: String) -> (U128, [u8; 20]) {
        let owner = env::predecessor_account_id();
        let mut account = self.get_account(&owner);
        assert!(account.balance >= amount.0, "Not enough balance");
        account.balance -= amount.0;
        self.total_supply -= amount.0;
        self.set_account(&owner, &account);
        let recipient = hex::decode(recipient).expect("recipient should be a hex");
        assert_eq!(
            recipient.len(),
            20,
            "Recipient should be a 20-bytes long address"
        );
        let mut raw_recipient = [0u8; 20];
        raw_recipient.copy_from_slice(&recipient);
        (amount, raw_recipient)
    }

    /// Record proof to make sure it is not re-used later for minting.
    fn record_proof(&mut self, proof: &Proof) {
        let mut data = proof.log_index.try_to_vec().unwrap();
        data.extend(proof.receipt_index.try_to_vec().unwrap());
        data.extend(proof.header_data.clone());
        let key = env::sha256(&data);
        assert!(
            !self.used_events.contains(&key),
            "Event cannot be reused for minting."
        );
        self.used_events.insert(&key);
    }
}

impl MintableFungibleToken {
    /// Helper method to get the account details for `owner_id`.
    fn get_account(&self, owner_id: &AccountId) -> Account {
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "Owner's account ID is invalid"
        );
        let account_hash = env::sha256(owner_id.as_bytes());
        self.accounts
            .get(&account_hash)
            .unwrap_or_else(|| Account::new(account_hash))
    }

    /// Helper method to set the account details for `owner_id` to the state.
    fn set_account(&mut self, owner_id: &AccountId, account: &Account) {
        let account_hash = env::sha256(owner_id.as_bytes());
        if account.balance > 0 || !account.allowances.is_empty() {
            self.accounts.insert(&account_hash, &account);
        } else {
            self.accounts.remove(&account_hash);
        }
    }

    fn refund_storage(&self, initial_storage: StorageUsage) {
        let current_storage = env::storage_usage();
        let attached_deposit = env::attached_deposit();
        let refund_amount = if current_storage > initial_storage {
            let required_deposit =
                Balance::from(current_storage - initial_storage) * STORAGE_PRICE_PER_BYTE;
            assert!(
                required_deposit <= attached_deposit,
                "The required attached deposit is {}, but the given attached deposit is is {}",
                required_deposit,
                attached_deposit,
            );
            attached_deposit - required_deposit
        } else {
            attached_deposit
                + Balance::from(initial_storage - current_storage) * STORAGE_PRICE_PER_BYTE
        };
        if refund_amount > 0 {
            env::log(
                format!(
                    "Refunding {} tokens for storage to {}",
                    refund_amount,
                    env::signer_account_id()
                )
                .as_bytes(),
            );
            Promise::new(env::signer_account_id()).transfer(refund_amount);
        }
    }

    /// Helper constructor for tests to initialize this token with already given amount of tokens
    /// to test.
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(test)]
    fn new_with_supply(owner_id: AccountId, total_supply: U128) -> Self {
        let prover_account = "testprover".to_string();
        let locker_address = [0u8; 20];
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "Owner's account ID is invalid"
        );
        let total_supply = total_supply.into();
        assert!(!env::state_exists(), "Already initialized");
        let mut ft = Self {
            accounts: UnorderedMap::new(b"a".to_vec()),
            total_supply,
            prover_account,
            locker_address,
            used_events: UnorderedSet::new(b"u".to_vec()),
        };
        let mut account = ft.get_account(&owner_id);
        account.balance = total_supply;
        ft.set_account(&owner_id, &account);
        ft
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    use super::*;

    const BURN_AMOUNT: u128 = 1000;

    fn alice() -> AccountId {
        "alice.near".to_string()
    }
    fn bob() -> AccountId {
        "bob.near".to_string()
    }
    fn carol() -> AccountId {
        "carol.near".to_string()
    }
    fn rainbow_bridge_eth_on_near_prover() -> AccountId {
        "rainbow_bridge_eth_on_near_prover".to_string()
    }

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        VMContext {
            current_account_id: alice(),
            signer_account_id: bob(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 1_000_000_000_000_000_000_000_000_000u128,
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn test_new() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = 1_000_000_000_000_000u128;
        let contract = MintableFungibleToken::new_with_supply(bob(), total_supply.into());
        assert_eq!(contract.get_total_supply().0, total_supply);
        assert_eq!(contract.get_balance(bob()).0, total_supply);
    }

    #[test]
    #[should_panic]
    fn test_new_twice_fails() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = 1_000_000_000_000_000u128;
        {
            let _contract = MintableFungibleToken::new_with_supply(bob(), total_supply.into());
        }
        MintableFungibleToken::new_with_supply(bob(), total_supply.into());
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = MintableFungibleToken::new_with_supply(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();

        context.attached_deposit = 1000 * STORAGE_PRICE_PER_BYTE;
        testing_env!(context.clone());
        let transfer_amount = total_supply / 3;
        contract.transfer(bob(), transfer_amount.into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(
            contract.get_balance(carol()).0,
            (total_supply - transfer_amount)
        );
        assert_eq!(contract.get_balance(bob()).0, transfer_amount);
    }

    #[test]
    #[should_panic(expected = "The new owner should be different from the current owner")]
    fn test_transfer_fail_self() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = MintableFungibleToken::new_with_supply(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();

        context.attached_deposit = 1000 * STORAGE_PRICE_PER_BYTE;
        testing_env!(context.clone());
        let transfer_amount = total_supply / 3;
        contract.transfer(carol(), transfer_amount.into());
    }

    #[test]
    #[should_panic(expected = "Can not increment allowance for yourself")]
    fn test_self_inc_allowance_fail() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = MintableFungibleToken::new_with_supply(carol(), total_supply.into());
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.inc_allowance(carol(), (total_supply / 2).into());
    }

    #[test]
    #[should_panic(expected = "Can not decrement allowance for yourself")]
    fn test_self_dec_allowance_fail() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = MintableFungibleToken::new_with_supply(carol(), total_supply.into());
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.dec_allowance(carol(), (total_supply / 2).into());
    }

    #[test]
    fn test_saturating_dec_allowance() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = MintableFungibleToken::new_with_supply(carol(), total_supply.into());
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.dec_allowance(bob(), (total_supply / 2).into());
        assert_eq!(contract.get_allowance(carol(), bob()), 0.into())
    }

    #[test]
    fn test_saturating_inc_allowance() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = std::u128::MAX;
        let mut contract = MintableFungibleToken::new_with_supply(carol(), total_supply.into());
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.inc_allowance(bob(), total_supply.into());
        contract.inc_allowance(bob(), total_supply.into());
        assert_eq!(
            contract.get_allowance(carol(), bob()),
            std::u128::MAX.into()
        )
    }

    #[test]
    #[should_panic(
        expected = "The required attached deposit is 33100000000000000000000, but the given attached deposit is is 0"
    )]
    fn test_self_allowance_fail_no_deposit() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = MintableFungibleToken::new_with_supply(carol(), total_supply.into());
        context.attached_deposit = 0;
        testing_env!(context.clone());
        contract.inc_allowance(bob(), (total_supply / 2).into());
    }

    #[test]
    fn test_carol_escrows_to_bob_transfers_to_alice() {
        // Acting as carol
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = MintableFungibleToken::new_with_supply(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();

        context.is_view = true;
        testing_env!(context.clone());
        assert_eq!(contract.get_total_supply().0, total_supply);

        let allowance = total_supply / 3;
        let transfer_amount = allowance / 3;
        context.is_view = false;
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.inc_allowance(bob(), allowance.into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(contract.get_allowance(carol(), bob()).0, allowance);

        // Acting as bob now
        context.is_view = false;
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        context.predecessor_account_id = bob();
        testing_env!(context.clone());
        contract.transfer_from(carol(), alice(), transfer_amount.into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(
            contract.get_balance(carol()).0,
            total_supply - transfer_amount
        );
        assert_eq!(contract.get_balance(alice()).0, transfer_amount);
        assert_eq!(
            contract.get_allowance(carol(), bob()).0,
            allowance - transfer_amount
        );
    }

    #[test]
    fn test_carol_escrows_to_bob_locks_and_transfers_to_alice() {
        // Acting as carol
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = MintableFungibleToken::new_with_supply(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();

        context.is_view = true;
        testing_env!(context.clone());
        assert_eq!(contract.get_total_supply().0, total_supply);

        let allowance = total_supply / 3;
        let transfer_amount = allowance / 3;
        context.is_view = false;
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.inc_allowance(bob(), allowance.into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(contract.get_allowance(carol(), bob()).0, allowance);
        assert_eq!(contract.get_balance(carol()).0, total_supply);

        // Acting as bob now
        context.is_view = false;
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        context.predecessor_account_id = bob();
        testing_env!(context.clone());
        contract.transfer_from(carol(), alice(), transfer_amount.into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(
            contract.get_balance(carol()).0,
            (total_supply - transfer_amount)
        );
        assert_eq!(contract.get_balance(alice()).0, transfer_amount);
        assert_eq!(
            contract.get_allowance(carol(), bob()).0,
            allowance - transfer_amount
        );
    }

    #[test]
    fn test_self_allowance_set_for_refund() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = MintableFungibleToken::new_with_supply(carol(), total_supply.into());
        context.storage_usage = env::storage_usage();

        let initial_balance = context.account_balance;
        let initial_storage = context.storage_usage;
        context.attached_deposit = STORAGE_PRICE_PER_BYTE * 1000;
        testing_env!(context.clone());
        contract.inc_allowance(bob(), (total_supply / 2).into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();
        assert_eq!(
            context.account_balance,
            initial_balance
                + Balance::from(context.storage_usage - initial_storage) * STORAGE_PRICE_PER_BYTE
        );

        let initial_balance = context.account_balance;
        let initial_storage = context.storage_usage;
        testing_env!(context.clone());
        context.attached_deposit = 0;
        testing_env!(context.clone());
        contract.dec_allowance(bob(), (total_supply / 2).into());
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();
        assert!(context.storage_usage < initial_storage);
        assert!(context.account_balance < initial_balance);
        assert_eq!(
            context.account_balance,
            initial_balance
                - Balance::from(initial_storage - context.storage_usage) * STORAGE_PRICE_PER_BYTE
        );
    }

    fn burn_common(owner: AccountId, sender: AccountId, total_supply: u128, burn_amount: u128, eth_recipient: &str) {
        let mut context = get_context(owner.clone());
        testing_env!(context.clone());
        let mut contract = MintableFungibleToken::new_with_supply(owner.clone(), total_supply.into());
        
        context.storage_usage = env::storage_usage();
        context.attached_deposit = 1000 * STORAGE_PRICE_PER_BYTE;
        context.predecessor_account_id = sender;
        testing_env!(context.clone());

        contract.burn(
            burn_amount.into(),
            eth_recipient.into(),
        );
        assert_eq!(contract.get_balance(alice()).0, total_supply - 1000);
        assert_eq!(contract.get_total_supply().0, total_supply - 1000);
    }

    #[test]
    fn test_burn() {
        burn_common(alice(), alice(), 1_000_000_000_000_000u128, BURN_AMOUNT, "0123456789abcdef0123456789abcdef01234567")
    }

    #[test]
    #[should_panic(expected = "recipient should be a hex: OddLength")]
    fn test_burn_invalid_recepient() {
        burn_common(alice(), alice(), 1_000_000_000_000_000u128, BURN_AMOUNT, "0123456789abcdef0123456789abcdef0123456")
    }

    #[test]
    #[should_panic(expected = "Not enough balance")]
    fn test_burn_insufficient_balance() {
        burn_common(alice(), carol(), 1_000_000_000_000_000u128, BURN_AMOUNT, "0123456789abcdef0123456789abcdef01234567")
    }

    fn mint_common(owner: AccountId, sender: AccountId, receiver: AccountId, total_supply: u128, locker_address: [u8; 20], proof: Proof, double_mint: bool) {
        let mut context = get_context(owner.clone());
        testing_env!(context.clone());
        let mut contract = MintableFungibleToken::new_with_supply(owner, total_supply.into());
        contract.locker_address = locker_address;

        context.storage_usage = env::storage_usage();
        context.attached_deposit = 1000 * STORAGE_PRICE_PER_BYTE;
        context.predecessor_account_id = sender.clone();
        testing_env!(context.clone());

        assert_eq!(
            contract.get_balance(receiver.clone()).0,
            0
        );
        contract.mint(proof.clone());
        if double_mint {
            contract.mint(proof);
        }
        assert_eq!(
            contract.get_balance(receiver.clone()).0,
            1000
        );
        assert_eq!(contract.get_total_supply().0, total_supply + 1000);
    }

    fn proof_from_file(path: &str) -> Proof {
        serde_json::from_reader(
            std::fs::File::open(std::path::Path::new(path)).unwrap(),
        ).unwrap()
    }

    #[test]
    fn test_mint() {
        mint_common(alice(), alice(), rainbow_bridge_eth_on_near_prover(), 1_000_000_000_000_000u128, [
            196, 199, 73, 127, 190, 26, 136, 104, 65, 161, 149, 165, 214, 34, 205, 96, 5, 60, 19,
            118,
        ], proof_from_file("data/proof.json"), false);
    }

    #[test]
    #[should_panic(expected = "Event cannot be reused for minting.")]
    fn test_mint_no_double_mint() {
        mint_common(alice(), alice(), rainbow_bridge_eth_on_near_prover(), 1_000_000_000_000_000u128, [
            196, 199, 73, 127, 190, 26, 136, 104, 65, 161, 149, 165, 214, 34, 205, 96, 5, 60, 19,
            118,
        ], proof_from_file("data/proof.json"), true);
    }

    #[test]
    #[should_panic(expected = "does not match locker address of this token")]
    fn test_mint_wrong_locker_address() {
        mint_common(alice(), alice(), rainbow_bridge_eth_on_near_prover(), 1_000_000_000_000_000u128, [
            100; 20
        ], proof_from_file("data/proof.json"), false);
    }

    #[test]
    #[should_panic(
        expected = "Finish transfer is only allowed to be called by the contract itself"
    )]
    fn test_mint_wrong_sender() {
        mint_common(carol(), carol(), rainbow_bridge_eth_on_near_prover(), 1_000_000_000_000_000u128, [
            196, 199, 73, 127, 190, 26, 136, 104, 65, 161, 149, 165, 214, 34, 205, 96, 5, 60, 19,
            118,
        ], proof_from_file("data/proof.json"), false);
    }
}
