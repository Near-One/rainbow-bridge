//! Compatibility with eth2-client contracts before 0.8.0 (pre-Glamsterdam).
//!
//! Older contracts take a different Borsh layout in both submit calls:
//! - `submit_beacon_chain_light_client_update`: `LightClientHeader` carries the full execution
//!   payload header instead of the execution block hash;
//! - `submit_execution_header`: `BlockHeader` has no Amsterdam fields.
//!
//! The relayer checks the contract version on every iteration and switches to these types, so it
//! can be deployed before the contract upgrade and keeps working across it.
//!
//! TODO: remove this module and the `LEGACY(0.7)` branches once all contracts run 0.8.0+.

use borsh::BorshSerialize;
use color_eyre::{Result, eyre};
use eth_types::eth2::{BeaconBlockHeader, ExecutionPayloadHeader, SyncAggregate, SyncCommittee};
use eth_types::{Address, Bloom, H64, H256, U256};
use serde::{Deserialize, de::Error as _};

use crate::ContractClient;

/// The first contract version that takes the current format
const FIRST_CURRENT_VERSION: (u64, u64) = (0, 8);

pub async fn is_legacy_contract(near_client: &ContractClient) -> Result<bool> {
    is_legacy_version(&near_client.get_version().await?)
}

fn is_legacy_version(version: &str) -> Result<bool> {
    let mut parts = version.split('.').map(str::parse::<u64>);
    match (parts.next(), parts.next()) {
        (Some(Ok(major)), Some(Ok(minor))) => Ok((major, minor) < FIRST_CURRENT_VERSION),
        _ => Err(eyre::eyre!("Unexpected contract version: {version}")),
    }
}

/// `LightClientHeader` with the full execution payload header, as served by the beacon API
/// before Gloas
#[derive(Debug, Clone, BorshSerialize, Deserialize)]
pub struct LightClientHeader {
    pub beacon: BeaconBlockHeader,
    pub execution: ExecutionPayloadHeader,
    pub execution_branch: Vec<H256>,
}

#[derive(Debug, Clone, BorshSerialize)]
pub struct LightClientUpdate {
    pub attested_header: LightClientHeader,
    pub next_sync_committee: Option<SyncCommittee>,
    pub next_sync_committee_branch: Option<Vec<H256>>,
    pub finalized_header: LightClientHeader,
    pub finality_branch: Vec<H256>,
    pub sync_aggregate: SyncAggregate,
    pub signature_slot: u64,
}

impl<'de> Deserialize<'de> for LightClientUpdate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let header =
            |name: &str| LightClientHeader::deserialize(&value[name]).map_err(D::Error::custom);
        let attested_header = header("attested_header")?;
        let finalized_header = header("finalized_header")?;

        // All other fields are unchanged, so they are parsed by the current type
        let update: eth_types::eth2::LightClientUpdate =
            serde_json::from_value(value).map_err(D::Error::custom)?;

        Ok(Self {
            attested_header,
            next_sync_committee: update.next_sync_committee,
            next_sync_committee_branch: update.next_sync_committee_branch,
            finalized_header,
            finality_branch: update.finality_branch,
            sync_aggregate: update.sync_aggregate,
            signature_slot: update.signature_slot,
        })
    }
}

/// `BlockHeader` without the Amsterdam fields
#[derive(Debug, Clone, BorshSerialize)]
pub struct BlockHeader {
    pub parent_hash: H256,
    pub sha3_uncles: H256,
    pub miner: Address,
    pub state_root: H256,
    pub transactions_root: H256,
    pub receipts_root: H256,
    pub logs_bloom: Bloom,
    pub difficulty: U256,
    pub number: u64,
    pub gas_limit: U256,
    pub gas_used: U256,
    pub timestamp: u64,
    pub extra_data: Vec<u8>,
    pub mix_hash: H256,
    pub nonce: H64,
    pub base_fee_per_gas: Option<u64>,
    pub withdrawals_root: Option<H256>,
    pub blob_gas_used: Option<u64>,
    pub excess_blob_gas: Option<u64>,
    pub parent_beacon_block_root: Option<H256>,
    pub requests_hash: Option<H256>,
    pub hash: Option<H256>,
    pub partial_hash: Option<H256>,
}

impl TryFrom<&eth_types::BlockHeader> for BlockHeader {
    type Error = color_eyre::Report;

    fn try_from(header: &eth_types::BlockHeader) -> Result<Self> {
        if header.block_access_list_hash.is_some() || header.slot_number.is_some() {
            return Err(eyre::eyre!(
                "Block {} is an Amsterdam block, the contract must be upgraded to accept it",
                header.number
            ));
        }

        Ok(Self {
            parent_hash: header.parent_hash,
            sha3_uncles: header.sha3_uncles,
            miner: header.miner,
            state_root: header.state_root,
            transactions_root: header.transactions_root,
            receipts_root: header.receipts_root,
            logs_bloom: header.logs_bloom,
            difficulty: header.difficulty,
            number: header.number,
            gas_limit: header.gas_limit,
            gas_used: header.gas_used,
            timestamp: header.timestamp,
            extra_data: header.extra_data.clone(),
            mix_hash: header.mix_hash,
            nonce: header.nonce,
            base_fee_per_gas: header.base_fee_per_gas,
            withdrawals_root: header.withdrawals_root,
            blob_gas_used: header.blob_gas_used,
            excess_blob_gas: header.excess_blob_gas,
            parent_beacon_block_root: header.parent_beacon_block_root,
            requests_hash: header.requests_hash,
            hash: header.hash,
            partial_hash: header.partial_hash,
        })
    }
}

pub fn to_legacy_headers(headers: &[eth_types::BlockHeader]) -> Result<Vec<BlockHeader>> {
    headers.iter().map(BlockHeader::try_from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> T {
        serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap()
    }

    #[test]
    fn test_is_legacy_version() {
        assert!(is_legacy_version("0.7.1").unwrap());
        assert!(!is_legacy_version("0.8.0").unwrap());
        assert!(!is_legacy_version("1.0.0").unwrap());
        assert!(is_legacy_version("garbage").is_err());
    }

    #[test]
    fn test_light_client_update_keeps_execution_payload() {
        let path = "./tests/data/light_client_update_period_925.json";
        let legacy: LightClientUpdate = read_json(path);
        let current: eth_types::eth2::LightClientUpdate = read_json(path);

        for (legacy, current) in [
            (&legacy.attested_header, &current.attested_header),
            (&legacy.finalized_header, &current.finalized_header),
        ] {
            assert_eq!(legacy.execution.block_hash, current.execution_block_hash);
            // The original branch from the beacon API, not the one converted to the Gloas format
            assert_eq!(legacy.execution_branch.len(), 4);
            assert!(current.execution_branch.ends_with(&legacy.execution_branch));
        }
    }

    #[test]
    fn test_block_header_layout() {
        let headers: Vec<eth_types::BlockHeader> =
            read_json("./tests/data/execution_blocks_8286935_8295112.json");
        let header = &headers[0];

        // The legacy layout is the current one without the two `None` Amsterdam fields,
        // which are serialized as one zero byte each right before `hash` and `partial_hash`
        let current = borsh::to_vec(header).unwrap();
        let tail_len = borsh::to_vec(&(&header.hash, &header.partial_hash))
            .unwrap()
            .len();
        let split = current.len() - tail_len;
        assert_eq!(&current[split - 2..split], &[0, 0]);
        let expected = [&current[..split - 2], &current[split..]].concat();

        let legacy = borsh::to_vec(&BlockHeader::try_from(header).unwrap()).unwrap();
        assert_eq!(legacy, expected);
    }

    #[test]
    fn test_amsterdam_block_header_is_rejected() {
        let mut headers: Vec<eth_types::BlockHeader> =
            read_json("./tests/data/execution_blocks_8286935_8295112.json");
        headers[1].slot_number = Some(1);
        assert!(to_legacy_headers(&headers).is_err());
    }
}
