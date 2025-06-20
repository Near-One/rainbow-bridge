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

// New combined header structure
#[derive(Debug, Clone, BorshDeserialize, BorshSchema, BorshSerialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct LightClientHeader {
    pub beacon: BeaconBlockHeader,
    pub execution: ExecutionPayloadHeader,
    pub execution_branch: Vec<H256>,
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
            execution_block_hash: finalized_header.execution.block_hash,
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
    use ssz_types::{typenum::U32, VariableList};
    use tree_hash::TreeHash;

    #[test]
    fn test_extra_data_tree_hash_matches_variable_list() {
        // Test cases with different lengths
        let test_cases = vec![
            vec![],                         // Empty
            vec![0x42],                     // Single byte
            vec![0x01, 0x02, 0x03, 0x04],   // Few bytes
            vec![0xff; 16],                 // Half capacity
            vec![0xaa; 32],                 // Full capacity
            (0u8..32).collect::<Vec<u8>>(), // Sequential bytes 0-31
        ];

        for test_data in test_cases {
            // Create ExtraData
            let extra_data = ExtraData(test_data.clone());

            // Create equivalent VariableList<u8, U32>
            let variable_list: VariableList<u8, U32> = VariableList::from(test_data.clone());

            // Compare tree hashes
            let extra_data_hash = extra_data.tree_hash_root();
            let variable_list_hash = variable_list.tree_hash_root();

            assert_eq!(
                extra_data_hash, variable_list_hash,
                "Tree hashes should match for data: {:?}",
                test_data
            );

            println!(
                "✓ Length {}: ExtraData and VariableList<u8, U32> produce same hash: 0x{}",
                test_data.len(),
                hex::encode(extra_data_hash)
            );
        }
    }

    #[test]
    fn test_extra_data_tree_hash_specific_values() {
        // Test some specific known patterns
        let test_cases = vec![
            (vec![0x00], "single zero byte"),
            (vec![0xff], "single max byte"),
            (
                vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
                "8 bytes hex pattern",
            ),
            (b"hello".to_vec(), "ascii text"),
        ];

        for (data, description) in test_cases {
            let extra_data = ExtraData(data.clone());
            let variable_list: VariableList<u8, U32> = VariableList::from(data.clone());

            assert_eq!(
                extra_data.tree_hash_root(),
                variable_list.tree_hash_root(),
                "Failed for test case: {}",
                description
            );
        }
    }

    #[test]
    fn test_extra_data_tree_hash_properties() {
        // Test that different lengths produce different hashes
        let data1 = ExtraData(vec![0x42]);
        let data2 = ExtraData(vec![0x42, 0x42]);

        assert_ne!(
            data1.tree_hash_root(),
            data2.tree_hash_root(),
            "Different lengths should produce different hashes"
        );

        // Test that different content produces different hashes
        let data3 = ExtraData(vec![0x01, 0x02, 0x03]);
        let data4 = ExtraData(vec![0x03, 0x02, 0x01]);

        assert_ne!(
            data3.tree_hash_root(),
            data4.tree_hash_root(),
            "Different content should produce different hashes"
        );
    }

    #[test]
    fn test_extra_data_empty_vs_zero_bytes() {
        // Empty data vs data with zero bytes should be different
        let empty = ExtraData(vec![]);
        let one_zero = ExtraData(vec![0x00]);
        let two_zeros = ExtraData(vec![0x00, 0x00]);

        let empty_hash = empty.tree_hash_root();
        let one_zero_hash = one_zero.tree_hash_root();
        let two_zeros_hash = two_zeros.tree_hash_root();

        assert_ne!(
            empty_hash, one_zero_hash,
            "Empty vs one zero byte should differ"
        );
        assert_ne!(
            empty_hash, two_zeros_hash,
            "Empty vs two zero bytes should differ"
        );
        assert_ne!(
            one_zero_hash, two_zeros_hash,
            "One vs two zero bytes should differ"
        );

        // Verify against VariableList
        let empty_vl: VariableList<u8, U32> = VariableList::from(vec![]);
        let one_zero_vl: VariableList<u8, U32> = VariableList::from(vec![0x00]);
        let two_zeros_vl: VariableList<u8, U32> = VariableList::from(vec![0x00, 0x00]);

        assert_eq!(empty_hash, empty_vl.tree_hash_root());
        assert_eq!(one_zero_hash, one_zero_vl.tree_hash_root());
        assert_eq!(two_zeros_hash, two_zeros_vl.tree_hash_root());
    }
}
