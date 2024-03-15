use crate::{
    primitives::*,
    serde::{hex_to_int, hex_to_int_opt},
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    pub parent_hash: U256,
    pub sha3_uncles: U256,
    pub miner: Address,
    pub state_root: U256,
    pub transactions_root: U256,
    pub receipts_root: U256,
    pub logs_bloom: Bloom,
    #[serde(deserialize_with = "hex_to_int")]
    pub difficulty: u128,
    pub number: Bytes,
    pub gas_limit: Bytes,
    pub gas_used: Bytes,
    pub timestamp: Bytes,
    pub extra_data: Bytes,
    pub mix_hash: U256,
    pub nonce: Bytes,
    #[serde(default, deserialize_with = "hex_to_int_opt")]
    pub base_fee_per_gas: Option<u64>,
    pub withdrawals_root: Option<U256>,
    #[serde(default, deserialize_with = "hex_to_int_opt")]
    pub blob_gas_used: Option<u64>,
    #[serde(default, deserialize_with = "hex_to_int_opt")]
    pub excess_blob_gas: Option<u64>,
    pub parent_beacon_block_root: Option<U256>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub address: Address,
    pub topics: Vec<U256>,
    pub data: Bytes,
    #[serde(deserialize_with = "hex_to_int")]
    pub log_index: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    #[serde(deserialize_with = "hex_to_int")]
    pub block_number: u64,
    #[serde(deserialize_with = "hex_to_int")]
    pub transaction_index: u64,
    #[serde(rename = "type", deserialize_with = "hex_to_int")]
    pub transaction_type: u8,
    pub cumulative_gas_used: Bytes,
    pub logs_bloom: Bloom,
    pub logs: Vec<Log>,
    #[serde(deserialize_with = "hex_to_int")]
    pub status: u8,
}
