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

#[cfg(not(target_arch = "wasm32"))]
impl ExecutionPayloadHeader {
    /// Position of `block_hash` among the fields of the execution payload header
    const BLOCK_HASH_INDEX: usize = 12;

    /// Merkle branch (bottom-up) proving `block_hash` within the execution payload header.
    fn block_hash_branch(&self) -> Vec<H256> {
        let mut leaves: Vec<tree_hash::Hash256> = vec![
            self.parent_hash.tree_hash_root(),
            self.fee_recipient.tree_hash_root(),
            self.state_root.tree_hash_root(),
            self.receipts_root.tree_hash_root(),
            self.logs_bloom.tree_hash_root(),
            self.prev_randao.tree_hash_root(),
            self.block_number.tree_hash_root(),
            self.gas_limit.tree_hash_root(),
            self.gas_used.tree_hash_root(),
            self.timestamp.tree_hash_root(),
            self.extra_data.tree_hash_root(),
            self.base_fee_per_gas.tree_hash_root(),
            self.block_hash.tree_hash_root(),
            self.transactions_root.tree_hash_root(),
        ];

        // Capella+
        if let Some(withdrawals_root) = self.withdrawals_root {
            leaves.push(withdrawals_root.tree_hash_root());
        }

        // Deneb+
        if self.blob_gas_used.is_some() || self.excess_blob_gas.is_some() {
            leaves.push(self.blob_gas_used.unwrap_or_default().tree_hash_root());
            leaves.push(self.excess_blob_gas.unwrap_or_default().tree_hash_root());
        }

        let mut layer = leaves;
        layer.resize(
            layer.len().next_power_of_two(),
            tree_hash::Hash256::default(),
        );

        let mut index = Self::BLOCK_HASH_INDEX;
        let mut branch = vec![];
        while layer.len() > 1 {
            branch.push(H256(layer[index ^ 1].0.into()));
            layer = layer
                .chunks(2)
                .map(|pair| tree_hash::merkle_root(&[pair[0].0, pair[1].0].concat(), 2))
                .collect();
            index /= 2;
        }

        branch
    }
}

// Light client header in the Gloas format, used for all forks. Pre-Gloas headers served by the
// beacon API (with the full `execution` payload header) are converted on deserialization, the same
// way as `upgrade_lc_header_to_gloas` does: the branch then proves the block hash through the
// execution payload header.
// Spec: https://github.com/ethereum/consensus-specs/blob/dev/specs/gloas/light-client/sync-protocol.md#modified-lightclientheader
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[cfg_attr(not(target_arch = "wasm32"), serde(try_from = "LightClientHeaderJson"))]
pub struct LightClientHeader {
    pub beacon: BeaconBlockHeader,
    // Since Gloas (EIP-7732) it's `parent_block_hash` of the payload bid, i.e. the hash of the
    // latest revealed execution block this beacon block builds on, not the payload of the same slot
    pub execution_block_hash: H256,
    pub execution_branch: Vec<H256>,
}

/// Light client header as served by the beacon API, in either the pre-Gloas or the Gloas format
#[cfg(not(target_arch = "wasm32"))]
#[derive(Deserialize)]
struct LightClientHeaderJson {
    beacon: BeaconBlockHeader,
    #[serde(default)]
    execution: Option<ExecutionPayloadHeader>,
    #[serde(default)]
    execution_block_hash: Option<H256>,
    execution_branch: Vec<H256>,
}

#[cfg(not(target_arch = "wasm32"))]
impl TryFrom<LightClientHeaderJson> for LightClientHeader {
    type Error = String;

    fn try_from(header: LightClientHeaderJson) -> Result<Self, Self::Error> {
        match (header.execution, header.execution_block_hash) {
            (None, Some(execution_block_hash)) => Ok(Self {
                beacon: header.beacon,
                execution_block_hash,
                execution_branch: header.execution_branch,
            }),
            (Some(execution), None) => {
                let mut execution_branch = execution.block_hash_branch();
                execution_branch.extend(header.execution_branch);
                Ok(Self {
                    beacon: header.beacon,
                    execution_block_hash: execution.block_hash,
                    execution_branch,
                })
            }
            _ => Err(
                "Light client header must contain exactly one of `execution` and `execution_block_hash`"
                    .to_string(),
            ),
        }
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
        let beacon = finalized_header.beacon;
        Self {
            header: beacon.clone(),
            beacon_block_root: beacon.tree_hash_root().0.into(),
            execution_block_hash: finalized_header.execution_block_hash,
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

    fn read_update_json(period: u64) -> serde_json::Value {
        let path = format!(
            "{}/../eth2-client/src/data/sepolia/light_client_update_period_{}.json",
            env!("CARGO_MANIFEST_DIR"),
            period
        );
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn test_pre_gloas_header_conversion() {
        let json = read_update_json(925);
        let expected_hash: H256 =
            serde_json::from_value(json["finalized_header"]["execution"]["block_hash"].clone())
                .unwrap();

        let header: LightClientHeader =
            serde_json::from_value(json["finalized_header"].clone()).unwrap();

        assert_eq!(header.execution_block_hash, expected_hash);
        // Block hash within the Deneb execution payload header (5) + payload header within
        // the beacon block body (4)
        assert_eq!(header.execution_branch.len(), 9);
        assert_eq!(
            header.execution_branch[5..],
            serde_json::from_value::<Vec<H256>>(
                json["finalized_header"]["execution_branch"].clone()
            )
            .unwrap()[..]
        );
    }

    #[test]
    fn test_gloas_header_serde_roundtrip() {
        let json = read_update_json(1379);
        let header: LightClientHeader =
            serde_json::from_value(json["finalized_header"].clone()).unwrap();
        assert_eq!(header.execution_branch.len(), 11);

        let serialized = serde_json::to_value(&header).unwrap();
        assert_eq!(serialized, json["finalized_header"]);
    }

    #[test]
    fn test_light_client_header_requires_exactly_one_format() {
        let mut json = read_update_json(925)["finalized_header"].clone();
        json["execution_block_hash"] = json["execution"]["block_hash"].clone();
        assert!(serde_json::from_value::<LightClientHeader>(json.clone()).is_err());

        json.as_object_mut().unwrap().remove("execution");
        json.as_object_mut().unwrap().remove("execution_block_hash");
        assert!(serde_json::from_value::<LightClientHeader>(json).is_err());
    }

    #[test]
    fn test_light_client_update_borsh_roundtrip() {
        let update: LightClientUpdate = serde_json::from_value(read_update_json(1379)).unwrap();
        let decoded = LightClientUpdate::try_from_slice(&borsh::to_vec(&update).unwrap()).unwrap();
        assert_eq!(
            decoded.finalized_header.execution_block_hash,
            update.finalized_header.execution_block_hash
        );
        assert_eq!(
            decoded.finalized_header.execution_branch,
            update.finalized_header.execution_branch
        );
    }
}
