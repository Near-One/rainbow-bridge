use eth_types::eth2::{BeaconBlockHeader, SyncCommittee};
use eth_types::H256;
use serde::Serialize;

#[derive(Serialize)]
pub struct LightClientSnapshotWithProof {
    pub beacon_header: BeaconBlockHeader,
    pub current_sync_committee: SyncCommittee,
    pub current_sync_committee_branch: Vec<H256>,
}
