use super::*;
use borsh::{BorshDeserialize, BorshSerialize};
use std::io::{Error, Write};
use tree_hash::MerkleHasher;

#[cfg(not(target_arch = "wasm32"))]
use {
    hex::FromHex,
    serde::{Deserialize, Deserializer, Serialize, Serializer},
};

pub const PUBLIC_KEY_BYTES_LEN: usize = 48;
pub const SIGNATURE_BYTES_LEN: usize = 96;
pub const SYNC_COMMITTEE_BITS_SIZE_IN_BYTES: usize = 512 / 8;

pub type Slot = u64;
pub type Epoch = u64;
pub type ForkVersion = [u8; 4];
pub type DomainType = [u8; 4];

#[derive(Debug, Clone, BorshSchema)]
pub struct PublicKeyBytes(pub [u8; PUBLIC_KEY_BYTES_LEN]);
#[derive(Debug, Clone, BorshSchema)]
pub struct SignatureBytes(pub [u8; SIGNATURE_BYTES_LEN]);
#[derive(Debug, Clone, BorshSchema)]
pub struct SyncCommitteeBits(pub [u8; SYNC_COMMITTEE_BITS_SIZE_IN_BYTES]);

arr_wrapper_impl_tree_hash_and_borsh!(PublicKeyBytes, PUBLIC_KEY_BYTES_LEN);
arr_wrapper_impl_tree_hash_and_borsh!(SignatureBytes, SIGNATURE_BYTES_LEN);
arr_wrapper_impl_tree_hash_and_borsh!(SyncCommitteeBits, SYNC_COMMITTEE_BITS_SIZE_IN_BYTES);

#[derive(
    Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize, tree_hash_derive::TreeHash,
)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct BeaconBlockHeader {
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub slot: Slot,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body_root: H256,
}

#[derive(Debug, Clone, PartialEq, tree_hash_derive::TreeHash)]
pub struct ForkData {
    pub current_version: ForkVersion,
    pub genesis_validators_root: H256,
}

#[derive(Debug, PartialEq, Clone, tree_hash_derive::TreeHash)]
pub struct SigningData {
    pub object_root: H256,
    pub domain: H256,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct ExtendedBeaconBlockHeader {
    pub header: BeaconBlockHeader,
    pub beacon_block_root: H256,
    pub execution_block_hash: H256,
}

impl From<HeaderUpdate> for ExtendedBeaconBlockHeader {
    fn from(item: HeaderUpdate) -> Self {
        let root = item.beacon_header.tree_hash_root();
        ExtendedBeaconBlockHeader {
            header: item.beacon_header,
            beacon_block_root: H256(root.0.into()),
            execution_block_hash: item.execution_block_hash,
        }
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct SyncCommitteePublicKeys(pub Vec<PublicKeyBytes>);
vec_wrapper_impl_tree_hash!(SyncCommitteePublicKeys);

#[derive(
    Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize, tree_hash_derive::TreeHash,
)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct SyncCommittee {
    pub pubkeys: SyncCommitteePublicKeys,
    pub aggregate_pubkey: PublicKeyBytes,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct SyncAggregate {
    pub sync_committee_bits: SyncCommitteeBits,
    pub sync_committee_signature: SignatureBytes,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct SyncCommitteeUpdate {
    pub next_sync_committee: SyncCommittee,
    pub next_sync_committee_branch: Vec<H256>,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct HeaderUpdate {
    pub beacon_header: BeaconBlockHeader,
    pub execution_block_hash: H256,
    pub execution_hash_branch: Vec<H256>,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct FinalizedHeaderUpdate {
    pub header_update: HeaderUpdate,
    pub finality_branch: Vec<H256>,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct LightClientUpdate {
    pub attested_beacon_header: BeaconBlockHeader,
    pub sync_aggregate: SyncAggregate,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub signature_slot: Slot,
    pub finality_update: FinalizedHeaderUpdate,
    pub sync_committee_update: Option<SyncCommitteeUpdate>,
}

#[derive(Clone, BorshDeserialize, BorshSchema, BorshSerialize, Debug)]
pub struct LightClientState {
    pub finalized_beacon_header: ExtendedBeaconBlockHeader,
    pub current_sync_committee: SyncCommittee,
    pub next_sync_committee: SyncCommittee,
}
