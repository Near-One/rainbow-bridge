use super::*;
use borsh::{BorshDeserialize, BorshSerialize};
use std::io::{Error, Write};
use tree_hash::{MerkleHasher, TreeHash};

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
        // Replicate the logic from VariableList<u8, U32>
        // ExtraData has a maximum length of 32 bytes (U32)
        const MAX_LEN: usize = 32;

        // For u8 (basic type), calculate number of leaves based on packing factor
        // u8 has a packing factor of 32 (32 bytes fit in one 32-byte chunk)
        let packing_factor = <u8 as tree_hash::TreeHash>::tree_hash_packing_factor();
        let num_leaves = (MAX_LEN + packing_factor - 1) / packing_factor;

        let mut hasher = tree_hash::MerkleHasher::with_leaves(num_leaves);

        for item in &self.0 {
            hasher
                .write(&item.tree_hash_packed_encoding())
                .expect("ExtraData should not contain more elements than max");
        }

        let root = hasher
            .finish()
            .expect("ExtraData should not have a remaining buffer");

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

// Execution header structure supporting multiple forks
// Spec: https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb/beacon-chain.md#executionpayloadheader
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct ExecutionPayloadHeader {
    // Core fields present since The Merge
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
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "quoted_u256"))]
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    pub transactions_root: H256,

    // Optional field introduced in Shanghai/Capella fork
    pub withdrawals_root: Option<H256>,

    // Optional fields introduced in Cancun/Deneb fork (EIP-4844 blob transactions)
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "optional_quoted_u64"))]
    pub blob_gas_used: Option<u64>,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "optional_quoted_u64"))]
    pub excess_blob_gas: Option<u64>,
}

// Light client header supporting both pre-Gloas and Gloas formats
// Spec: https://github.com/ethereum/consensus-specs/blob/dev/specs/gloas/light-client/sync-protocol.md#modified-lightclientheader
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct LightClientHeader {
    pub beacon: BeaconBlockHeader,
    // Pre-Gloas format (Capella..Fulu)
    #[cfg_attr(
        not(target_arch = "wasm32"),
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub execution: Option<ExecutionPayloadHeader>,
    // Gloas format (EIP-7732): `parent_block_hash` of the payload bid, i.e. the hash of
    // the latest revealed execution block this beacon block builds on, not the payload
    // of the same slot
    #[cfg_attr(
        not(target_arch = "wasm32"),
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub execution_block_hash: Option<H256>,
    pub execution_branch: Vec<H256>,
}

impl LightClientHeader {
    /// Returns the execution block hash regardless of the header format.
    pub fn execution_block_hash(&self) -> Option<H256> {
        self.execution_block_hash.or_else(|| {
            self.execution
                .as_ref()
                .map(|execution| execution.block_hash)
        })
    }
}

#[derive(Debug, Clone, PartialEq, tree_hash_derive::TreeHash)]
pub struct ForkData {
    pub current_version: ForkVersion,
    pub genesis_validators_root: H256,
}

/// This is used specifically for backwards-compatibility, storing state in the contract
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct ExtendedBeaconBlockHeader {
    pub header: BeaconBlockHeader,
    pub beacon_block_root: H256,
    pub execution_block_hash: H256,
}

impl From<LightClientHeader> for ExtendedBeaconBlockHeader {
    fn from(finalized_header: LightClientHeader) -> Self {
        let execution_block_hash = finalized_header
            .execution_block_hash()
            .expect("LightClientHeader has no execution block hash");
        let beacon = finalized_header.beacon;
        Self {
            header: beacon.clone(),
            beacon_block_root: beacon.tree_hash_root().0.into(),
            execution_block_hash,
        }
    }
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
// Spec: https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientupdate
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct LightClientUpdate {
    pub attested_header: LightClientHeader,
    pub next_sync_committee: Option<SyncCommittee>,
    pub next_sync_committee_branch: Option<Vec<H256>>,
    pub finalized_header: LightClientHeader,
    pub finality_branch: Vec<H256>,
    pub sync_aggregate: SyncAggregate,
    #[cfg_attr(not(target_arch = "wasm32"), serde(with = "serde_utils::quoted_u64"))]
    pub signature_slot: Slot,
}

// For arrays of light client updates
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct LightClientUpdates(pub Vec<LightClientUpdate>);

#[derive(Clone, BorshDeserialize, BorshSchema, BorshSerialize, Debug)]
pub struct LightClientState {
    pub finalized_beacon_header: ExtendedBeaconBlockHeader,
    pub current_sync_committee: SyncCommittee,
    pub next_sync_committee: SyncCommittee,
}

/// Serde module for handling `Option<u64>` with quotes
pub mod optional_quoted_u64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => serializer.serialize_some(&format!("{}", v)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => s.parse::<u64>().map(Some).map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

/// Serde module for handling `U256` with quotes
pub mod quoted_u256 {
    use crate::U256;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", value))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        // Parse as decimal (base 10)
        ethereum_types::U256::from_dec_str(&s)
            .map(U256)
            .map_err(|e| {
                serde::de::Error::custom(format!("failed to parse U256 from decimal: {}", e))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_pre_gloas_update() -> LightClientUpdate {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../eth2-client/src/data/sepolia/light_client_update_period_925.json"
        );
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn to_gloas_format(header: &LightClientHeader) -> LightClientHeader {
        LightClientHeader {
            beacon: header.beacon.clone(),
            execution: None,
            execution_block_hash: header.execution_block_hash(),
            execution_branch: header.execution_branch.clone(),
        }
    }

    #[test]
    fn test_pre_gloas_light_client_header() {
        let header = load_pre_gloas_update().finalized_header;
        let execution = header.execution.as_ref().unwrap();

        assert!(header.execution_block_hash.is_none());
        assert_eq!(header.execution_block_hash(), Some(execution.block_hash));

        let json = serde_json::to_value(&header).unwrap();
        assert!(json.get("execution_block_hash").is_none());
    }

    #[test]
    fn test_gloas_light_client_header_serde() {
        let pre_gloas = load_pre_gloas_update().finalized_header;
        let block_hash = pre_gloas.execution.as_ref().unwrap().block_hash;
        let header = to_gloas_format(&pre_gloas);

        let json = serde_json::to_value(&header).unwrap();
        assert!(json.get("execution").is_none());

        let parsed: LightClientHeader = serde_json::from_value(json).unwrap();
        assert!(parsed.execution.is_none());
        assert_eq!(parsed.execution_block_hash(), Some(block_hash));

        let extended: ExtendedBeaconBlockHeader = parsed.into();
        assert_eq!(extended.execution_block_hash, block_hash);
    }

    #[test]
    fn test_light_client_update_borsh_roundtrip() {
        let mut update = load_pre_gloas_update();
        let block_hash = update.finalized_header.execution_block_hash();

        let decoded = LightClientUpdate::try_from_slice(&borsh::to_vec(&update).unwrap()).unwrap();
        assert_eq!(decoded.finalized_header.execution_block_hash(), block_hash);

        update.finalized_header = to_gloas_format(&update.finalized_header);
        let decoded = LightClientUpdate::try_from_slice(&borsh::to_vec(&update).unwrap()).unwrap();
        assert!(decoded.finalized_header.execution.is_none());
        assert_eq!(decoded.finalized_header.execution_block_hash(), block_hash);
    }

    #[test]
    #[should_panic(expected = "LightClientHeader has no execution block hash")]
    fn test_light_client_header_without_execution_block_hash() {
        let mut header = load_pre_gloas_update().finalized_header;
        header.execution = None;
        let _: ExtendedBeaconBlockHeader = header.into();
    }
}
