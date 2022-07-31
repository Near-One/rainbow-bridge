use std::collections::HashMap;
use std::str::FromStr;

use admin_controlled::Mask;
use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use borsh::{BorshDeserialize, BorshSerialize};
use eth2_utility::consensus::*;
use eth2_utility::types::*;
use eth_types::eth2::*;
use eth_types::{BlockHeader, H256};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::{assert_self, env, near_bindgen, AccountId, PanicOnDefault};
use near_sdk_inner::collections::LazyOption;
use near_sdk_inner::{Balance, BorshStorageKey, Promise};
use tree_hash::TreeHash;

#[cfg(test)]
mod tests;

const PAUSE_SUBMIT_UPDATE: Mask = 1;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    FinalizedExecutionBlocks,
    UnfinalizedHeaders,
    Submitters,
    FinalizedExecutionHeader,
    CurrentSyncCommittee,
    NextSyncCommittee,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct EthClient {
    /// If set, only light client updates by the trusted signer will be accepted
    trusted_signer: Option<AccountId>,
    /// Mask determining all paused functions
    paused: Mask,
    /// Whether the client validates the updates.
    /// Should only be set to `false` for debugging, testing, and diagnostic purposes
    validate_updates: bool,
    /// Whether the client verifies BLS signatures.
    verify_bls_signatures: bool,
    /// We store the hashes of the blocks for the past `hashes_gc_threshold` headers.
    /// Events that happen past this threshold cannot be verified by the client.
    /// It is desirable that this number is larger than 7 days' worth of headers, which is roughly
    /// 51k Ethereum blocks. So this number should be 51k in production.
    hashes_gc_threshold: u64,
    /// Network. e.g. mainnet, kiln
    network: Network,
    /// Hashes of the finalized execution blocks mapped to their numbers. Stores up to `hashes_gc_threshold` entries.
    /// Execution block number -> execution block hash
    finalized_execution_blocks: LookupMap<u64, H256>,
    /// All unfinalized execution blocks' headers hashes mapped to their `HeaderInfo`.
    /// Execution block hash -> ExecutionHeaderInfo object
    unfinalized_headers: UnorderedMap<H256, ExecutionHeaderInfo>,
    /// `AccountId`s mapped to their number of submitted headers.
    /// Submitter account -> Num of submitted headers
    submitters: LookupMap<AccountId, u32>,
    /// Max number of unfinalized blocks allowed to be stored by one submitter account
    /// This value should be at least 32 blocks (1 epoch), but the recommended value is 1024 (32 epochs)
    max_submitted_blocks_by_account: u32,
    // The minimum balance that should be attached to register a new submitter account
    min_storage_balance_for_submitter: Balance,
    /// Light client state
    finalized_beacon_header: ExtendedBeaconBlockHeader,
    finalized_execution_header: LazyOption<ExecutionHeaderInfo>,
    current_sync_committee: LazyOption<SyncCommittee>,
    next_sync_committee: LazyOption<SyncCommittee>,
}

#[near_bindgen]
impl EthClient {
    #[init]
    pub fn init(#[serializer(borsh)] args: InitInput) -> Self {
        assert!(!Self::initialized(), "Already initialized");
        let min_storage_balance_for_submitter =
            calculate_min_storage_balance_for_submitter(args.max_submitted_blocks_by_account);
        let network =
            Network::from_str(args.network.as_str()).unwrap_or_else(|e| env::panic_str(e.as_str()));

        if network == Network::Mainnet {
            assert!(
                args.validate_updates,
                "The updates validation can't be disabled for mainnet"
            );
        }

        assert_eq!(
            args.finalized_execution_header.calculate_hash(),
            args.finalized_beacon_header.execution_block_hash,
            "Invalid execution block"
        );

        let finalized_execution_header_info = ExecutionHeaderInfo {
            parent_hash: args.finalized_execution_header.parent_hash,
            block_number: args.finalized_execution_header.number,
            submitter: env::predecessor_account_id(),
        };

        Self {
            trusted_signer: args.trusted_signer,
            paused: Mask::default(),
            validate_updates: args.validate_updates,
            verify_bls_signatures: args.verify_bls_signatures,
            hashes_gc_threshold: args.hashes_gc_threshold,
            network,
            finalized_execution_blocks: LookupMap::new(StorageKey::FinalizedExecutionBlocks),
            unfinalized_headers: UnorderedMap::new(StorageKey::UnfinalizedHeaders),
            submitters: LookupMap::new(StorageKey::Submitters),
            max_submitted_blocks_by_account: args.max_submitted_blocks_by_account,
            min_storage_balance_for_submitter,
            finalized_beacon_header: args.finalized_beacon_header,
            finalized_execution_header: LazyOption::new(
                StorageKey::FinalizedExecutionHeader,
                Some(&finalized_execution_header_info),
            ),
            current_sync_committee: LazyOption::new(
                StorageKey::CurrentSyncCommittee,
                Some(&args.current_sync_committee),
            ),
            next_sync_committee: LazyOption::new(
                StorageKey::NextSyncCommittee,
                Some(&args.next_sync_committee),
            ),
        }
    }

    #[result_serializer(borsh)]
    pub fn initialized() -> bool {
        env::state_read::<EthClient>().is_some()
    }

    /// Returns finalized execution block number
    #[result_serializer(borsh)]
    pub fn last_block_number(&self) -> u64 {
        self.finalized_execution_header.get().unwrap().block_number
    }

    /// Returns finalized execution block hash
    #[result_serializer(borsh)]
    pub fn block_hash_safe(&self, #[serializer(borsh)] block_number: u64) -> Option<H256> {
        self.finalized_execution_blocks.get(&block_number)
    }

    /// Checks if the execution header is already submitted.
    #[result_serializer(borsh)]
    pub fn is_known_execution_header(&self, #[serializer(borsh)] hash: H256) -> bool {
        self.unfinalized_headers.get(&hash).is_some()
    }

    /// Get finalized beacon block root
    #[result_serializer(borsh)]
    pub fn finalized_beacon_block_root(&self) -> H256 {
        self.finalized_beacon_header.beacon_block_root
    }

    /// Returns finalized beacon block slot
    #[result_serializer(borsh)]
    pub fn finalized_beacon_block_slot(&self) -> u64 {
        self.finalized_beacon_header.header.slot
    }

    /// Returns finalized beacon block header
    #[result_serializer(borsh)]
    pub fn finalized_beacon_block_header(&self) -> ExtendedBeaconBlockHeader {
        self.finalized_beacon_header.clone()
    }

    /// Returns the minimum balance that should be attached to register a new submitter account
    #[result_serializer(borsh)]
    pub fn min_storage_balance_for_submitter(&self) -> Balance {
        self.min_storage_balance_for_submitter
    }

    /// Get the current light client state
    #[result_serializer(borsh)]
    pub fn get_light_client_state(&self) -> LightClientState {
        LightClientState {
            finalized_beacon_header: self.finalized_beacon_header.clone(),
            current_sync_committee: self.current_sync_committee.get().unwrap(),
            next_sync_committee: self.next_sync_committee.get().unwrap(),
        }
    }

    #[payable]
    pub fn register_submitter(&mut self) {
        let account_id = env::predecessor_account_id();
        assert!(
            !self.submitters.contains_key(&account_id),
            "The account is already registered"
        );

        let amount = env::attached_deposit();
        assert!(
            amount >= self.min_storage_balance_for_submitter,
            "{}",
            format!(
                "The attached deposit {} is less than the minimum required storage balance {}",
                amount, self.min_storage_balance_for_submitter
            )
        );

        self.submitters.insert(&account_id, &0);
        let refund = amount
            .checked_sub(self.min_storage_balance_for_submitter)
            .unwrap();
        if refund > 0 {
            Promise::new(account_id).transfer(refund);
        }
    }

    #[payable]
    pub fn unregister_submitter(&mut self) -> Promise {
        let account_id = env::predecessor_account_id();
        if let Some(num_of_submitted_blocks) = self.submitters.remove(&account_id) {
            if num_of_submitted_blocks > 0 {
                env::panic_str("Can't unregister the account with used storage")
            }

            Promise::new(account_id).transfer(self.min_storage_balance_for_submitter)
        } else {
            env::panic_str("The account is not registered");
        }
    }

    pub fn submit_beacon_chain_light_client_update(
        &mut self,
        #[serializer(borsh)] update: LightClientUpdate,
    ) {
        self.is_light_client_update_allowed();

        if self.validate_updates {
            self.validate_light_client_update(&update);
        }

        self.commit_light_client_update(update);
    }

    #[result_serializer(borsh)]
    pub fn submit_execution_header(&mut self, #[serializer(borsh)] block_header: BlockHeader) {
        #[cfg(feature = "logs")]
        env::log_str(format!("Submitted header number {}", block_header.number).as_str());
        if self.finalized_beacon_header.execution_block_hash != block_header.parent_hash {
            self.unfinalized_headers
                .get(&block_header.parent_hash)
                .unwrap_or_else(|| {
                    panic!(
                        "Header has unknown parent {:?}. Parent should be submitted first.",
                        block_header.parent_hash
                    )
                });
        }

        let submitter = env::predecessor_account_id();
        self.update_submitter(&submitter, 1);
        let block_hash = block_header.calculate_hash();
        #[cfg(feature = "logs")]
        env::log_str(format!("Submitted header hash {:?}", block_hash).as_str());

        let block_info = ExecutionHeaderInfo {
            parent_hash: block_header.parent_hash,
            block_number: block_header.number,
            submitter,
        };
        let insert_result = self.unfinalized_headers.insert(&block_hash, &block_info);
        assert!(
            insert_result.is_none(),
            "The block {} already submitted!",
            &block_hash
        );
    }

    pub fn update_trusted_signer(&mut self, trusted_signer: Option<AccountId>) {
        assert_self();
        self.trusted_signer = trusted_signer;
    }

    pub fn get_trusted_signer(&self) -> Option<AccountId> {
        self.trusted_signer.clone()
    }
}

impl EthClient {
    fn validate_light_client_update(&self, update: &LightClientUpdate) {
        #[cfg(feature = "logs")]
        env::log_str(format!("Validate update. Used gas: {}", env::used_gas().0).as_str());

        let finalized_period =
            compute_sync_committee_period(self.finalized_beacon_header.header.slot);
        self.verify_finality_branch(update, finalized_period);

        // Verify sync committee has sufficient participants
        let sync_committee_bits =
            BitVec::<u8, Lsb0>::from_slice(&update.sync_aggregate.sync_committee_bits.0);
        let sync_committee_bits_sum: u64 = sync_committee_bits.count_ones().try_into().unwrap();

        assert!(
            sync_committee_bits_sum >= MIN_SYNC_COMMITTEE_PARTICIPANTS,
            "Invalid sync committee bits sum: {}",
            sync_committee_bits_sum
        );
        assert!(
            sync_committee_bits_sum * 3 >= (sync_committee_bits.len() * 2).try_into().unwrap(),
            "Sync committee bits sum is less than 2/3 threshold, bits sum: {}",
            sync_committee_bits_sum
        );

        #[cfg(feature = "bls")]
        if self.verify_bls_signatures {
            self.verify_bls_signatures(update, sync_committee_bits, finalized_period);
        }

        #[cfg(feature = "logs")]
        env::log_str(format!("Finish validate update. Used gas: {}", env::used_gas().0).as_str());
    }

    fn verify_finality_branch(&self, update: &LightClientUpdate, finalized_period: u64) {
        // The active header will always be the finalized header because we don't accept updates without the finality update.
        let active_header = &update.finality_update.header_update.beacon_header;

        assert!(
            active_header.slot > self.finalized_beacon_header.header.slot,
            "The active header slot number should be higher than the finalized slot"
        );

        let update_period = compute_sync_committee_period(active_header.slot);
        assert!(
            update_period == finalized_period || update_period == finalized_period + 1,
            "The acceptable update periods are '{}' and '{}' but got {}",
            finalized_period,
            finalized_period + 1,
            update_period
        );

        // Verify that the `finality_branch`, confirms `finalized_header`
        // to match the finalized checkpoint root saved in the state of `attested_header`.
        let branch = convert_branch(&update.finality_update.finality_branch);
        assert!(
            merkle_proof::verify_merkle_proof(
                update
                    .finality_update
                    .header_update
                    .beacon_header
                    .tree_hash_root(),
                &branch,
                FINALITY_TREE_DEPTH.try_into().unwrap(),
                FINALITY_TREE_INDEX.try_into().unwrap(),
                update.attested_beacon_header.state_root.0
            ),
            "Invalid finality proof"
        );
        assert!(
            validate_beacon_block_header_update(&update.finality_update.header_update),
            "Invalid execution block hash proof"
        );

        // Verify that the `next_sync_committee`, if present, actually is the next sync committee saved in the
        // state of the `active_header`
        if update_period != finalized_period {
            let sync_committee_update = update
                .sync_committee_update
                .as_ref()
                .unwrap_or_else(|| env::panic_str("The sync committee update is missed"));
            let branch = convert_branch(&sync_committee_update.next_sync_committee_branch);
            assert!(
                merkle_proof::verify_merkle_proof(
                    sync_committee_update.next_sync_committee.tree_hash_root(),
                    &branch,
                    SYNC_COMMITTEE_TREE_DEPTH.try_into().unwrap(),
                    SYNC_COMMITTEE_TREE_INDEX.try_into().unwrap(),
                    active_header.state_root.0
                ),
                "Invalid next sync committee proof"
            );
        }
    }

    #[cfg(feature = "bls")]
    fn verify_bls_signatures(
        &self,
        update: &LightClientUpdate,
        sync_committee_bits: BitVec<u8>,
        finalized_period: u64,
    ) {
        let config = NetworkConfig::new(&self.network);
        let signature_period = compute_sync_committee_period(update.signature_slot);
        // Verify sync committee aggregate signature
        let sync_committee = if signature_period == finalized_period {
            self.current_sync_committee.get().unwrap()
        } else {
            self.next_sync_committee.get().unwrap()
        };

        let participant_pubkeys =
            get_participant_pubkeys(&sync_committee.pubkeys.0, &sync_committee_bits);
        let fork_version = config
            .compute_fork_version_by_slot(update.signature_slot)
            .expect("Unsupported fork");
        let domain = compute_domain(
            DOMAIN_SYNC_COMMITTEE,
            fork_version,
            config.genesis_validators_root.into(),
        );
        let signing_root = compute_signing_root(
            eth_types::H256(update.attested_beacon_header.tree_hash_root()),
            domain,
        );

        let aggregate_signature =
            bls::AggregateSignature::deserialize(&update.sync_aggregate.sync_committee_signature.0)
                .unwrap();
        let pubkeys: Vec<bls::PublicKey> = participant_pubkeys
            .into_iter()
            .map(|x| bls::PublicKey::deserialize(&x.0).unwrap())
            .collect();
        assert!(
            aggregate_signature
                .fast_aggregate_verify(signing_root.0, &pubkeys.iter().collect::<Vec<_>>()),
            "Failed to verify the bls signature"
        );
    }

    fn update_finalized_header(&mut self, finalized_header: ExtendedBeaconBlockHeader) {
        #[cfg(feature = "logs")]
        env::log_str(format!("Update finalized header. Used gas: {}", env::used_gas().0).as_str());
        let finalized_execution_header_info = self
            .unfinalized_headers
            .get(&finalized_header.execution_block_hash)
            .expect("Unknown execution block hash");

        #[cfg(feature = "logs")]
        env::log_str(
            format!(
                "Current finalized slot: {}, New finalized slot: {}",
                self.finalized_beacon_header.header.slot, finalized_header.header.slot
            )
            .as_str(),
        );

        let mut cursor_header = finalized_execution_header_info.clone();
        let mut cursor_header_hash = finalized_header.execution_block_hash;

        let mut submitters_update: HashMap<AccountId, u32> = HashMap::new();
        loop {
            let num_of_removed_headers = *submitters_update
                .get(&cursor_header.submitter)
                .unwrap_or(&0);
            submitters_update.insert(cursor_header.submitter, num_of_removed_headers + 1);

            self.unfinalized_headers.remove(&cursor_header_hash);
            self.finalized_execution_blocks
                .insert(&cursor_header.block_number, &cursor_header_hash);

            if cursor_header.parent_hash == self.finalized_beacon_header.execution_block_hash {
                break;
            }

            cursor_header_hash = cursor_header.parent_hash;
            cursor_header = self
                .unfinalized_headers
                .get(&cursor_header.parent_hash)
                .unwrap_or_else(|| {
                    panic!(
                        "Header has unknown parent {:?}. Parent should be submitted first.",
                        cursor_header.parent_hash
                    )
                });
        }
        self.finalized_beacon_header = finalized_header;
        self.finalized_execution_header
            .set(&finalized_execution_header_info);

        for (submitter, num_of_removed_headers) in &submitters_update {
            self.update_submitter(submitter, -(*num_of_removed_headers as i64));
        }

        #[cfg(feature = "logs")]
        env::log_str(
            format!(
                "Finish update finalized header. Used gas: {}",
                env::used_gas().0
            )
            .as_str(),
        );

        if finalized_execution_header_info.block_number > self.hashes_gc_threshold {
            self.gc_headers(
                finalized_execution_header_info.block_number - self.hashes_gc_threshold,
            );
        }
    }

    fn commit_light_client_update(&mut self, update: LightClientUpdate) {
        // Update finalized header
        let finalized_header_update = update.finality_update.header_update;
        let finalized_period =
            compute_sync_committee_period(self.finalized_beacon_header.header.slot);
        let update_period =
            compute_sync_committee_period(finalized_header_update.beacon_header.slot);

        if update_period == finalized_period + 1 {
            self.current_sync_committee
                .set(&self.next_sync_committee.get().unwrap());
            self.next_sync_committee
                .set(&update.sync_committee_update.unwrap().next_sync_committee);
        }

        self.update_finalized_header(finalized_header_update.into());
    }

    /// Remove information about the headers that are at least as old as the given block number.
    fn gc_headers(&mut self, mut header_number: u64) {
        loop {
            if self.finalized_execution_blocks.contains_key(&header_number) {
                self.finalized_execution_blocks.remove(&header_number);

                if header_number == 0 {
                    break;
                } else {
                    header_number -= 1;
                }
            } else {
                break;
            }
        }
    }

    fn update_submitter(&mut self, submitter: &AccountId, value: i64) {
        let mut num_of_submitted_headers: i64 = self
            .submitters
            .get(submitter)
            .unwrap_or_else(|| {
                env::panic_str(
                    format!(
                        "The account {} can't submit blocks because it is not registered",
                        &submitter
                    )
                    .as_str(),
                )
            })
            .into();

        num_of_submitted_headers += value;

        assert!(
            num_of_submitted_headers <= self.max_submitted_blocks_by_account.into(),
            "{}",
            format!(
                "The submitter {} exhausted the limit of blocks ({})",
                &submitter, self.max_submitted_blocks_by_account
            ),
        );

        self.submitters
            .insert(submitter, &num_of_submitted_headers.try_into().unwrap());
    }

    fn is_light_client_update_allowed(&self) {
        self.check_not_paused(PAUSE_SUBMIT_UPDATE);

        if let Some(trusted_signer) = &self.trusted_signer {
            assert_eq!(
                &env::signer_account_id(),
                trusted_signer,
                "Eth-client is deployed as trust mode, only trusted_signer can update the client"
            );
        }
    }
}

admin_controlled::impl_admin_controlled!(EthClient, paused);
