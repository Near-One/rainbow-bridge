use crate::{
    eth_rpc_client::EthRPCClient,
    primitives::*,
    types::{BlockHeader, Log, TransactionReceipt},
};
use cita_trie::{MemoryDB, PatriciaTrie, Trie, TrieError};
use hasher::HasherKeccak;
use rlp::RlpStream;
use std::error::Error;
use std::sync::Arc;

#[derive(Debug)]
pub struct Proof {
    pub log_index: u64,
    pub log_entry_data: Vec<u8>,
    pub receipt_index: u64,
    pub receipt_data: Vec<u8>,
    pub header_data: Vec<u8>,
    pub proof: Vec<Vec<u8>>,
}

pub fn get_proof_for_event(tx_hash: &str, log_index: u64, node_url: &str,) -> Result<Proof, Box<dyn Error>> {
    let client = EthRPCClient::new(node_url);

    let tx_bytes: [u8; 32] = hex::decode(&tx_hash[2..])
        .expect("Invalid hex string")
        .try_into()
        .expect("Invalid hex string");

    let receipt = client.get_transaction_receipt_by_hash(&FixedBytes(tx_bytes))?;
    let block_header = client.get_block_by_number(receipt.block_number)?;
    let block_receipts = client.get_block_receipts(receipt.block_number)?;

    let trie = build_receipt_trie(&block_receipts)?;

    let receipt_key = rlp::encode(&receipt.transaction_index);
    let proof = trie.get_proof(&receipt_key)?;

    let mut log_data: Option<Vec<u8>> = None;
    for (_, log) in receipt.logs.iter().enumerate() {
        if log.log_index == log_index {
            log_data = Some(encode_log(log));
        }
    }

    Ok(Proof {
        log_index,
        log_entry_data: log_data.expect("Log not found"),
        receipt_index: receipt.transaction_index,
        receipt_data: encode_receipt(&receipt),
        header_data: encode_header(&block_header),
        proof,
    })
}

fn build_receipt_trie(receipts: &[TransactionReceipt],) -> Result<PatriciaTrie<MemoryDB, HasherKeccak>, TrieError> {
    let memdb = Arc::new(MemoryDB::new(true));
    let hasher = Arc::new(HasherKeccak::new());
    let mut trie = PatriciaTrie::new(Arc::clone(&memdb), Arc::clone(&hasher));

    for (_, receipt) in receipts.iter().enumerate() {
        let receipt_key = rlp::encode(&receipt.transaction_index).to_vec();
        let receipt_data = encode_receipt(receipt);

        trie.insert(receipt_key, receipt_data)?;
    }

    Ok(trie)
}

fn encode_receipt(receipt: &TransactionReceipt) -> Vec<u8> {
    let mut stream = RlpStream::new();

    if receipt.transaction_type != 0 {
        stream.append(&receipt.transaction_type);
    }

    stream.begin_list(4);
    stream
        .append(&receipt.status)
        .append(&receipt.cumulative_gas_used.0)
        .append(&receipt.logs_bloom.0.to_vec());

    stream.begin_list(receipt.logs.len());
    for (_, log) in receipt.logs.iter().enumerate() {
        stream.begin_list(3);
        stream.append(&log.address.0.to_vec());

        stream.begin_list(log.topics.len());
        for (_, topic) in log.topics.iter().enumerate() {
            stream.append(&topic.0.to_vec());
        }

        stream.append(&log.data.0);
    }

    stream.out().to_vec()
}

fn encode_log(log: &Log) -> Vec<u8> {
    let mut stream = RlpStream::new();
    stream.begin_list(3);

    stream.append(&log.address.0.to_vec());

    stream.begin_list(log.topics.len());
    for (_, topic) in log.topics.iter().enumerate() {
        stream.append(&topic.0.to_vec());
    }

    stream.append(&log.data.0);

    stream.out().to_vec()
}

fn encode_header(header: &BlockHeader) -> Vec<u8> {
    let mut stream = RlpStream::new();
    stream.begin_unbounded_list();

    stream
        .append(&header.parent_hash.0.to_vec())
        .append(&header.sha3_uncles.0.to_vec())
        .append(&header.miner.0.to_vec())
        .append(&header.state_root.0.to_vec())
        .append(&header.transactions_root.0.to_vec())
        .append(&header.receipts_root.0.to_vec())
        .append(&header.logs_bloom.0.to_vec())
        .append(&header.difficulty)
        .append(&header.number.0)
        .append(&header.gas_limit.0)
        .append(&header.gas_used.0)
        .append(&header.timestamp.0)
        .append(&header.extra_data.0.to_vec())
        .append(&header.mix_hash.0.to_vec())
        .append(&header.nonce.0.to_vec());

    if header.base_fee_per_gas.is_some() {
        stream.append(&header.base_fee_per_gas.unwrap());
    }

    if header.withdrawals_root.is_some() {
        stream.append(&header.withdrawals_root.clone().unwrap().0.to_vec());
    }

    if header.blob_gas_used.is_some() {
        stream.append(&header.blob_gas_used.unwrap());
    }

    if header.excess_blob_gas.is_some() {
        stream.append(&header.excess_blob_gas.unwrap());
    }

    if header.parent_beacon_block_root.is_some() {
        stream.append(&header.parent_beacon_block_root.clone().unwrap().0.to_vec());
    }

    stream.finalize_unbounded_list();

    stream.out().to_vec()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use hasher::Hasher;

    #[test]
    fn generate_proof() {
        const RPC_URL: &'static str = "https://eth.llamarpc.com";
        const TX_HASH: &'static str = "0x9298954a9db8026ca28bce4d71ffb7ba0aac70e91f0667ffb7398c67e60b84fa";
        let proof = get_proof_for_event(TX_HASH, 377, RPC_URL).unwrap();

        let hasher = HasherKeccak::new();

        println!("Header {:x?}", hasher.digest(&proof.header_data));
        println!("Proof {:x?}", &proof.proof);
    }
}
