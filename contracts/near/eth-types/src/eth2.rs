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

#[derive(Debug, Clone, BorshSchema)]
pub struct ExtraData(pub Vec<u8>);

impl tree_hash::TreeHash for ExtraData {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        tree_hash::TreeHashType::List
    }

    fn tree_hash_packed_encoding(&self) -> tree_hash::PackedEncoding {
        unreachable!("List should never be packed.")
    }

    fn tree_hash_packing_factor() -> usize {
        unreachable!("List should never be packed.")
    }

    fn tree_hash_root(&self) -> tree_hash::Hash256 {
        let mut hasher =
            tree_hash::MerkleHasher::with_leaves(self.0.len().div_ceil(tree_hash::BYTES_PER_CHUNK));

        for item in &self.0 {
            hasher.write(&item.tree_hash_packed_encoding()).unwrap();
        }

        let root = hasher.finish().unwrap();
        tree_hash::mix_in_length(&root, self.0.len())
    }
}

// Add Borsh implementations
impl borsh::BorshSerialize for ExtraData {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.0, writer)
    }
}

impl borsh::BorshDeserialize for ExtraData {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        Ok(ExtraData(Vec::<u8>::deserialize_reader(reader)?))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl serde::Serialize for ExtraData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Always serialize as hex string
        let hex_string = format!("0x{}", hex::encode(&self.0));
        serializer.serialize_str(&hex_string)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<'de> serde::Deserialize<'de> for ExtraData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex_string = <std::string::String as Deserialize>::deserialize(deserializer)?;
        let hex_string = hex_string.strip_prefix("0x").unwrap_or(&hex_string);
        let bytes = hex::decode(hex_string)
            .map_err(|e| serde::de::Error::custom(format!("Invalid hex: {}", e)))?;
        Ok(ExtraData(bytes))
    }
}

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

// New execution header structure for Electra
#[derive(
    Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize, tree_hash_derive::TreeHash,
)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct ExecutionHeader {
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    pub logs_bloom: Bloom,
    pub prev_randao: H256,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub block_number: u64,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub gas_limit: u64,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub gas_used: u64,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub timestamp: u64,
    pub extra_data: ExtraData,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub base_fee_per_gas: u64,
    pub block_hash: H256,
    pub transactions_root: H256,
    pub withdrawals_root: H256,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub blob_gas_used: u64,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub excess_blob_gas: u64,
}

// New combined header structure
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct AttestedHeader {
    pub beacon: BeaconBlockHeader,
    pub execution: ExecutionHeader,
    pub execution_branch: Vec<H256>,
}

#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct FinalizedHeader {
    pub beacon: BeaconBlockHeader,
    pub execution: ExecutionHeader,
    pub execution_branch: Vec<H256>,
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

// Updated light client update structure for Electra
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct LightClientUpdate {
    pub attested_header: AttestedHeader,
    pub next_sync_committee: Option<SyncCommittee>,
    pub next_sync_committee_branch: Option<Vec<H256>>,
    pub finalized_header: FinalizedHeader,
    pub finality_branch: Vec<H256>,
    pub sync_aggregate: SyncAggregate,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub signature_slot: Slot,
}

// Version enum for different Ethereum fork versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[cfg_attr(not(target_arch = "wasm32"), serde(rename_all = "lowercase"))]
pub enum LightClientVersion {
    Altair,
    Bellatrix,
    Capella,
    Deneb,
    Electra,
}

impl LightClientVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            LightClientVersion::Altair => "altair",
            LightClientVersion::Bellatrix => "bellatrix",
            LightClientVersion::Capella => "capella",
            LightClientVersion::Deneb => "deneb",
            LightClientVersion::Electra => "electra",
        }
    }
}

impl std::fmt::Display for LightClientVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Top-level wrapper with version (optional, for when you need versioning)
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct VersionedLightClientUpdate {
    pub version: LightClientVersion,
    pub data: LightClientUpdate,
}

// For arrays of light client updates
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct LightClientUpdates(pub Vec<LightClientUpdate>);

// Alternative: Version-specific data if structures differ significantly between versions
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub enum LightClientUpdateVariant {
    Electra(LightClientUpdate),
    // Future versions can be added here with different data structures
    // Deneb(DenebLightClientUpdate),
}

#[derive(Clone, BorshDeserialize, BorshSchema, BorshSerialize, Debug)]
pub struct LightClientState {
    pub finalized_beacon_header: FinalizedHeader,
    pub current_sync_committee: SyncCommittee,
    pub next_sync_committee: SyncCommittee,
}
