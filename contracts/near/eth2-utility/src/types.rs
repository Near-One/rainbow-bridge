use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use eth_types::eth2::*;
use eth_types::H256;
use near_sdk::near;
use near_sdk::AccountId;

use crate::consensus::Network;

/// Minimal information about a header.
#[derive(Clone)]
#[near(serializers=[borsh])]
pub struct ExecutionHeaderInfo {
    pub parent_hash: H256,
    pub block_number: u64,
    pub submitter: AccountId,
}

#[derive(Clone)]
#[near(serializers=[borsh])]
pub struct InitInput {
    pub network: String,
    pub finalized_execution_header: eth_types::BlockHeader,
    pub finalized_beacon_header: ExtendedBeaconBlockHeader,
    pub current_sync_committee: SyncCommittee,
    pub next_sync_committee: SyncCommittee,
    pub validate_updates: bool,
    pub verify_bls_signatures: bool,
    pub hashes_gc_threshold: u64,
    pub trusted_signer: Option<AccountId>,
}

#[derive(Clone, BorshDeserialize, BorshSerialize, PartialEq, BorshSchema, Debug)]
pub enum ClientMode {
    SubmitLightClientUpdate,
    SubmitHeader,
}

#[derive(Clone)]
#[near(serializers=[json])]
pub struct ContractConfig {
    pub trusted_signer: Option<AccountId>,
    pub validate_updates: bool,
    pub verify_bls_signatures: bool,
    pub hashes_gc_threshold: u64,
    pub network: Network,
    pub trusted_blocks_submitter: Option<AccountId>,
}
