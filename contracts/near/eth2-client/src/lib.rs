use near_plugins::{
    access_control, access_control_any, pause, AccessControlRole, AccessControllable, Pausable,
    Upgradable,
};
use near_sdk::serde::{Deserialize, Serialize};

use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use borsh::{BorshDeserialize, BorshSerialize};
use eth2_utility::consensus::*;
use eth2_utility::types::*;
use eth_types::eth2::*;
use eth_types::{BlockHeader, H256};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{
    env, near, require, AccountId, BorshStorageKey, PanicOnDefault, Promise, PublicKey,
};
use tree_hash::TreeHash;

use amcl::bls381::bls381::utils::serialize_uncompressed_g1;
use amcl::bls381::ecp::ECP;
use amcl::bls381::fp2::FP2;
use amcl::bls381::hash_to_curve::hash_to_field_fp2;

mod migrate;
#[cfg(test)]
mod tests;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    FinalizedExecutionBlocks,
    __DeprecatedUnfinalizedHeaders,
    __DeprecatedSubmitters,
    FinalizedExecutionHeader,
    CurrentSyncCommittee,
    NextSyncCommittee,
}

#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    PauseManager,
    UpgradableCodeStager,
    UpgradableCodeDeployer,
    UnrestrictedSubmitLightClientUpdate,
    UnrestrictedSubmitExecutionHeader,
    DAO,
}

#[near(contract_state)]
#[derive(PanicOnDefault, Pausable, Upgradable)]
#[access_control(role_type(Role))]
#[pausable(manager_roles(Role::PauseManager, Role::DAO))]
#[upgradable(access_control_roles(
    code_stagers(Role::UpgradableCodeStager, Role::DAO),
    code_deployers(Role::UpgradableCodeDeployer, Role::DAO),
    duration_initializers(Role::DAO),
    duration_update_stagers(Role::DAO),
    duration_update_appliers(Role::DAO),
))]
pub struct Eth2Client {
    /// If set, only light client updates by the trusted signer will be accepted
    trusted_signer: Option<AccountId>,
    /// Mask determining all paused functions
    #[deprecated]
    paused: u128,
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
    /// Network. e.g. mainnet, goerli
    network: Network,
    /// Hashes of the finalized execution blocks mapped to their numbers. Stores up to `hashes_gc_threshold` entries.
    /// Execution block number -> execution block hash
    finalized_execution_blocks: LookupMap<u64, H256>,
    /// Light client state
    finalized_beacon_header: ExtendedBeaconBlockHeader,
    finalized_execution_header: LazyOption<ExecutionHeaderInfo>,
    current_sync_committee: LazyOption<SyncCommittee>,
    next_sync_committee: LazyOption<SyncCommittee>,
    client_mode: ClientMode,
    unfinalized_head_execution_header: Option<ExecutionHeaderInfo>,
    unfinalized_tail_execution_header: Option<ExecutionHeaderInfo>,
    trusted_blocks_submitter: Option<AccountId>,
}

#[near]
impl Eth2Client {
    #[init]
    #[private]
    pub fn init(#[serializer(borsh)] args: InitInput) -> Self {
        #[cfg(feature = "mainnet")]
        {
            require!(
                args.validate_updates,
                "The updates validation can't be disabled for mainnet"
            );

            require!(
                args.verify_bls_signatures
                    || args.trusted_signer.is_some(),
                "The client can't be executed in the trustless mode without BLS sigs verification on Mainnet"
            );
        }

        let finalized_execution_header_hash = args.finalized_execution_header.calculate_hash();

        require!(
            finalized_execution_header_hash == args.finalized_beacon_header.execution_block_hash,
            "Invalid execution block"
        );

        let finalized_execution_header_info = ExecutionHeaderInfo {
            parent_hash: args.finalized_execution_header.parent_hash,
            block_number: args.finalized_execution_header.number,
            submitter: env::predecessor_account_id(),
        };

        #[allow(deprecated)]
        let mut contract = Self {
            trusted_signer: args.trusted_signer,
            paused: 0,
            validate_updates: args.validate_updates,
            verify_bls_signatures: args.verify_bls_signatures,
            hashes_gc_threshold: args.hashes_gc_threshold,
            network: args.network,
            finalized_execution_blocks: LookupMap::new(StorageKey::FinalizedExecutionBlocks),
            finalized_beacon_header: args.finalized_beacon_header.into(),
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
            client_mode: ClientMode::SubmitLightClientUpdate,
            unfinalized_head_execution_header: None,
            unfinalized_tail_execution_header: None,
            trusted_blocks_submitter: None,
        };

        contract.finalized_execution_blocks.insert(
            &args.finalized_execution_header.number,
            &finalized_execution_header_hash,
        );

        contract.acl_init_super_admin(env::predecessor_account_id());
        contract
    }

    #[result_serializer(borsh)]
    pub fn initialized() -> bool {
        env::state_read::<Eth2Client>().is_some()
    }

    /// Returns finalized execution block number
    #[result_serializer(borsh)]
    pub fn last_block_number(&self) -> u64 {
        self.finalized_execution_header.get().unwrap().block_number
    }

    /// Returns finalized execution block hash
    #[result_serializer(borsh)]
    pub fn block_hash_safe(&self, #[serializer(borsh)] block_number: u64) -> Option<H256> {
        if block_number > self.finalized_execution_header.get().unwrap().block_number {
            return None;
        }
        self.finalized_execution_blocks.get(&block_number)
    }

    /// Checks if the execution header is already submitted.
    #[result_serializer(borsh)]
    pub fn is_known_execution_header(&self, block_number: u64) -> bool {
        self.finalized_execution_blocks.get(&block_number).is_some()
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

    /// Get the current light client state
    #[result_serializer(borsh)]
    pub fn get_light_client_state(&self) -> LightClientState {
        LightClientState {
            finalized_beacon_header: self.finalized_beacon_header.clone(),
            current_sync_committee: self.current_sync_committee.get().unwrap(),
            next_sync_committee: self.next_sync_committee.get().unwrap(),
        }
    }

    /// Returns current client mode
    #[result_serializer(borsh)]
    pub fn get_client_mode(&self) -> ClientMode {
        self.client_mode.clone()
    }

    /// Returns unfinalized tail execution block number
    #[result_serializer(borsh)]
    pub fn get_unfinalized_tail_block_number(&self) -> Option<u64> {
        self.unfinalized_tail_execution_header
            .as_ref()
            .map(|header| header.block_number)
    }

    #[pause(except(roles(Role::UnrestrictedSubmitLightClientUpdate, Role::DAO)))]
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
    #[pause(except(roles(Role::UnrestrictedSubmitExecutionHeader, Role::DAO)))]
    pub fn submit_execution_header(&mut self, #[serializer(borsh)] block_header: BlockHeader) {
        if let Some(trusted_blocks_submitter) = &self.trusted_blocks_submitter {
            require!(
                &env::predecessor_account_id() == trusted_blocks_submitter,
                "Eth-client is deployed as trust mode, only trusted_blocks_submitter can submit blocks"
            );
        }

        require!(
            self.client_mode == ClientMode::SubmitHeader,
            "Client is not in SubmitHeader mode"
        );

        let block_hash = block_header.calculate_hash();
        let expected_block_hash = self
            .unfinalized_tail_execution_header
            .as_ref()
            .map(|header| header.parent_hash)
            .unwrap_or(self.finalized_beacon_header.execution_block_hash);
        require!(
            block_hash == expected_block_hash,
            format!(
                "The expected block hash is {:#?} but got {:#?}.",
                expected_block_hash, block_hash
            )
        );

        let insert_result = self
            .finalized_execution_blocks
            .insert(&block_header.number, &block_hash);

        require!(
            insert_result.is_none(),
            format!("The block {:#?} already submitted!", &block_hash)
        );

        let finalized_execution_header = self.finalized_execution_header.get().unwrap();
        // Apply gc
        if let Some(diff_between_unfinalized_head_and_tail) =
            self.get_diff_between_unfinalized_head_and_tail()
        {
            let header_number_to_remove = (finalized_execution_header.block_number
                + diff_between_unfinalized_head_and_tail)
                .checked_sub(self.hashes_gc_threshold)
                .unwrap_or(0);

            require!(
                header_number_to_remove < finalized_execution_header.block_number,
                "The `hashes_gc_threshold` is not enough to be able to apply gc correctly"
            );

            if header_number_to_remove > 0 {
                self.gc_finalized_execution_blocks(header_number_to_remove);
            }
        }

        if block_header.number == finalized_execution_header.block_number + 1 {
            let finalized_execution_header_hash = self
                .finalized_execution_blocks
                .get(&finalized_execution_header.block_number)
                .unwrap();
            require!(
                block_header.parent_hash == finalized_execution_header_hash,
                "The chain cannot be closed"
            );

            #[cfg(feature = "logs")]
            env::log_str(
                format!(
                    "Current finalized block number: {}, New finalized block number: {}",
                    finalized_execution_header.block_number,
                    self.unfinalized_head_execution_header
                        .as_ref()
                        .unwrap()
                        .block_number
                )
                .as_str(),
            );

            self.finalized_execution_header
                .set(self.unfinalized_head_execution_header.as_ref().unwrap());
            self.unfinalized_tail_execution_header = None;
            self.unfinalized_head_execution_header = None;
            self.client_mode = ClientMode::SubmitLightClientUpdate;
        } else {
            let block_info = ExecutionHeaderInfo {
                parent_hash: block_header.parent_hash,
                block_number: block_header.number,
                submitter: env::predecessor_account_id(),
            };

            if self.unfinalized_head_execution_header.is_none() {
                self.unfinalized_head_execution_header = Some(block_info.clone());
            }
            self.unfinalized_tail_execution_header = Some(block_info);
        }

        #[cfg(feature = "logs")]
        env::log_str(
            format!(
                "Submitted header number {}, hash {:#?}",
                block_header.number, block_hash
            )
            .as_str(),
        );
    }

    #[access_control_any(roles(Role::DAO))]
    pub fn update_trusted_signer(&mut self, trusted_signer: Option<AccountId>) {
        self.trusted_signer = trusted_signer;
    }

    #[access_control_any(roles(Role::DAO))]
    pub fn update_trusted_blocks_submitter(&mut self, trusted_blocks_submitter: Option<AccountId>) {
        self.trusted_blocks_submitter = trusted_blocks_submitter;
    }

    #[access_control_any(roles(Role::DAO))]
    pub fn update_hashes_gc_threshold(&mut self, hashes_gc_threshold: u64) {
        self.hashes_gc_threshold = hashes_gc_threshold;
    }

    #[access_control_any(roles(Role::DAO))]
    pub fn attach_full_access_key(&self, public_key: PublicKey) -> Promise {
        Promise::new(env::current_account_id()).add_full_access_key(public_key)
    }

    #[access_control_any(roles(Role::DAO))]
    pub fn set_verify_bls_signatures(&mut self, enabled: bool) {
        self.verify_bls_signatures = enabled;
    }

    pub fn get_config(self) -> ContractConfig {
        ContractConfig {
            trusted_signer: self.trusted_signer,
            validate_updates: self.validate_updates,
            verify_bls_signatures: self.verify_bls_signatures,
            hashes_gc_threshold: self.hashes_gc_threshold,
            network: self.network,
            trusted_blocks_submitter: self.trusted_blocks_submitter,
        }
    }

    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }
}

impl Eth2Client {
    fn validate_light_client_update(&self, update: &LightClientUpdate) {
        let finalized_period =
            compute_sync_committee_period(self.finalized_beacon_header.header.slot);
        self.verify_finality_branch(update, finalized_period);

        // Verify sync committee has sufficient participants
        let sync_committee_bits =
            BitVec::<u8, Lsb0>::from_slice(&update.sync_aggregate.sync_committee_bits.0);
        let sync_committee_bits_sum: u64 = sync_committee_bits.count_ones().try_into().unwrap();

        require!(
            sync_committee_bits_sum >= MIN_SYNC_COMMITTEE_PARTICIPANTS,
            format!(
                "Invalid sync committee bits sum: {}",
                sync_committee_bits_sum
            )
        );

        require!(
            sync_committee_bits_sum * 3 >= (sync_committee_bits.len() * 2).try_into().unwrap(),
            format!(
                "Sync committee bits sum is less than 2/3 threshold, bits sum: {}",
                sync_committee_bits_sum
            )
        );

        if self.verify_bls_signatures {
            self.verify_bls_signatures(update, sync_committee_bits, finalized_period);
        }
    }

    fn verify_finality_branch(&self, update: &LightClientUpdate, finalized_period: u64) {
        // The active header will always be the finalized header because we don't accept updates without the finality update.
        let active_header = &update.finalized_header.beacon;

        require!(
            active_header.slot > self.finalized_beacon_header.header.slot,
            "The active header slot number should be higher than the finalized slot"
        );

        require!(
            update.attested_header.beacon.slot >= update.finalized_header.beacon.slot,
            "The attested header slot should be equal to or higher than the finalized header slot"
        );

        require!(
            update.signature_slot > update.attested_header.beacon.slot,
            "The signature slot should be higher than the attested header slot"
        );

        let update_period = compute_sync_committee_period(active_header.slot);
        require!(
            update_period == finalized_period || update_period == finalized_period + 1,
            format!(
                "The acceptable update periods are '{}' and '{}' but got {}",
                finalized_period,
                finalized_period + 1,
                update_period
            )
        );

        let config = NetworkConfig::new(&self.network);

        // Verify that the `finality_branch`, confirms `finalized_header`
        // to match the finalized checkpoint root saved in the state of `attested_header`.
        let generalized_index =
            config.get_generalized_index_constants(update.finalized_header.beacon.slot);
        require!(
            verify_merkle_proof(
                H256(update.finalized_header.beacon.tree_hash_root().0.into()),
                &update.finality_branch,
                generalized_index.finality_tree_depth.try_into().unwrap(),
                generalized_index.finality_tree_index.try_into().unwrap(),
                update.attested_header.beacon.state_root
            ),
            "Invalid finality proof"
        );

        require!(
            config.is_valid_light_client_header(&update.finalized_header),
            "Invalid execution block hash proof"
        );

        // Verify that the `next_sync_committee`, if present, actually is the next sync committee saved in the
        // state of the `active_header`
        if update_period != finalized_period {
            let generalized_index =
                config.get_generalized_index_constants(update.attested_header.beacon.slot);

            let sync_committee_update = update
                .next_sync_committee
                .as_ref()
                .unwrap_or_else(|| env::panic_str("The sync committee update is missed"));

            require!(
                verify_merkle_proof(
                    H256(sync_committee_update.tree_hash_root().0.into()),
                    &update.next_sync_committee_branch.clone().unwrap(),
                    generalized_index
                        .sync_committee_tree_depth
                        .try_into()
                        .unwrap(),
                    generalized_index
                        .sync_committee_tree_index
                        .try_into()
                        .unwrap(),
                    update.attested_header.beacon.state_root
                ),
                "Invalid next sync committee proof"
            );
        }
    }

    fn verify_bls_signatures(
        &self,
        update: &LightClientUpdate,
        sync_committee_bits: BitVec<u8>,
        finalized_period: u64,
    ) {
        let config = NetworkConfig::new(&self.network);
        let signature_period = compute_sync_committee_period(update.signature_slot);

        // Verify signature period does not skip a sync committee period
        require!(
            signature_period == finalized_period || signature_period == finalized_period + 1,
            format!(
                "The acceptable signature periods are '{}' and '{}' but got {}",
                finalized_period,
                finalized_period + 1,
                signature_period
            )
        );

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
            .unwrap_or_else(|| env::panic_str("Unsupported fork"));
        let domain = compute_domain(
            DOMAIN_SYNC_COMMITTEE,
            fork_version,
            config.genesis_validators_root.into(),
        );
        let signing_root = compute_signing_root(
            eth_types::H256(update.attested_header.beacon.tree_hash_root().0.into()),
            domain,
        );

        let msg_bytes = signing_root.0.as_bytes().to_vec();
        let signature_bytes = update.sync_aggregate.sync_committee_signature.0.to_vec();

        let dst: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
        let msg_fp2 = hash_to_field_fp2(msg_bytes.as_slice(), 2, dst).unwrap_or_else(|_| {
            env::panic_str("hash to field should not fail for given parameters")
        });

        let mut msg_fp2_0 = [0u8; 96];
        let mut msg_fp2_1 = [0u8; 96];
        Self::fp2_to_u8(&msg_fp2[0], &mut msg_fp2_0);
        Self::fp2_to_u8(&msg_fp2[1], &mut msg_fp2_1);

        let mut msg_g2_0 = env::bls12381_map_fp2_to_g2(&msg_fp2_0);
        let mut msg_g2_1 = env::bls12381_map_fp2_to_g2(&msg_fp2_1);
        let mut msg_g2_concat = vec![0u8; 1];
        msg_g2_concat.append(&mut msg_g2_0);
        msg_g2_concat.push(0);
        msg_g2_concat.append(&mut msg_g2_1);

        let msg_g2 = env::bls12381_p2_sum(&msg_g2_concat);

        let pubkeys_bytes: Vec<u8> = participant_pubkeys
            .iter()
            .flat_map(|x| x.0.iter())
            .copied()
            .collect();
        let pks_decompress = env::bls12381_p1_decompress(&pubkeys_bytes);
        let mut pks_with_sign = Vec::new();
        for chunk in pks_decompress.chunks(96) {
            pks_with_sign.push(0u8);
            pks_with_sign.extend_from_slice(chunk);
        }
        let pk_agg = env::bls12381_p1_sum(&pks_with_sign);

        let mut gen = ECP::generator();
        gen.neg();
        let gneg = serialize_uncompressed_g1(&gen);

        let sig_des = env::bls12381_p2_decompress(&signature_bytes);
        let pairing_input = [pk_agg, msg_g2, gneg.to_vec(), sig_des].concat();
        let ok = env::bls12381_pairing_check(&pairing_input);
        require!(ok, "Failed to verify the bls signature");
    }

    fn commit_light_client_update(&mut self, update: LightClientUpdate) {
        // Update finalized header
        let finalized_header_update = update.finalized_header;
        let finalized_period =
            compute_sync_committee_period(self.finalized_beacon_header.header.slot);
        let update_period = compute_sync_committee_period(finalized_header_update.beacon.slot);

        if update_period == finalized_period + 1 {
            self.current_sync_committee
                .set(&self.next_sync_committee.get().unwrap());
            self.next_sync_committee
                .set(&update.next_sync_committee.unwrap());
        }

        #[cfg(feature = "logs")]
        env::log_str(
            format!(
                "Current finalized slot: {}, New finalized slot: {}",
                self.finalized_beacon_header.header.slot, finalized_header_update.beacon.slot
            )
            .as_str(),
        );

        self.finalized_beacon_header = finalized_header_update.into();
        self.client_mode = ClientMode::SubmitHeader;
    }

    /// Remove information about the headers that are at least as old as the given block number.
    /// This method could go out of gas if the client was not synced for a while, to fix that
    /// you need to increase the `hashes_gc_threshold` by calling `update_hashes_gc_threshold()`
    fn gc_finalized_execution_blocks(&mut self, mut header_number: u64) {
        loop {
            if self
                .finalized_execution_blocks
                .remove(&header_number)
                .is_some()
            {
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

    fn is_light_client_update_allowed(&self) {
        require!(
            self.client_mode == ClientMode::SubmitLightClientUpdate,
            "Client is not in SubmitLightClientUpdate mode"
        );

        if let Some(trusted_signer) = &self.trusted_signer {
            require!(
                &env::predecessor_account_id() == trusted_signer,
                "Eth-client is deployed as trust mode, only trusted_signer can update the client"
            );
        }
    }

    fn get_diff_between_unfinalized_head_and_tail(&self) -> Option<u64> {
        let head_block_number = self
            .unfinalized_head_execution_header
            .as_ref()
            .map(|header| header.block_number)?;
        let tail_block_number = self
            .unfinalized_tail_execution_header
            .as_ref()
            .map(|header| header.block_number)?;

        Some(head_block_number - tail_block_number)
    }

    fn fp2_to_u8(u: &FP2, out: &mut [u8; 96]) {
        u.getb().to_byte_array(&mut out[0..48], 0);
        u.geta().to_byte_array(&mut out[48..96], 0);
    }
}
