use crate::*;
use near_sdk::store::UnorderedMap;
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Eth2ClientV1 {
    trusted_signer: Option<AccountId>,
    #[deprecated]
    paused: u128,
    validate_updates: bool,
    verify_bls_signatures: bool,
    hashes_gc_threshold: u64,
    network: Network,
    finalized_execution_blocks: LookupMap<u64, H256>,
    unfinalized_headers: UnorderedMap<H256, ExecutionHeaderInfo>,
    submitters: LookupMap<AccountId, u32>,
    max_submitted_blocks_by_account: u32,
    min_storage_balance_for_submitter: u128,
    finalized_beacon_header: ExtendedBeaconBlockHeader,
    finalized_execution_header: LazyOption<ExecutionHeaderInfo>,
    current_sync_committee: LazyOption<SyncCommittee>,
    next_sync_committee: LazyOption<SyncCommittee>,
}

#[near]
impl Eth2Client {
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old_state: Eth2ClientV1 = env::state_read().expect("failed");
        #[allow(deprecated)]
        Self {
            trusted_signer: old_state.trusted_signer,
            paused: old_state.paused,
            validate_updates: old_state.validate_updates,
            verify_bls_signatures: old_state.verify_bls_signatures,
            hashes_gc_threshold: old_state.hashes_gc_threshold,
            network: old_state.network,
            finalized_execution_blocks: old_state.finalized_execution_blocks,
            finalized_beacon_header: old_state.finalized_beacon_header,
            finalized_execution_header: old_state.finalized_execution_header,
            current_sync_committee: old_state.current_sync_committee,
            next_sync_committee: old_state.next_sync_committee,
            client_mode: ClientMode::SubmitLightClientUpdate,
            unfinalized_head_execution_header: None,
            unfinalized_tail_execution_header: None,
            trusted_blocks_submitter: None,
        }
    }
}
