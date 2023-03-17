use std::str::FromStr;

use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use borsh::{BorshDeserialize, BorshSerialize};
use eth_types::eth2::*;
use eth_types::H256;
use near_sdk::{env, Balance};
use tree_hash::TreeHash;

pub const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: u64 = 256;
pub const MIN_SYNC_COMMITTEE_PARTICIPANTS: u64 = 1;
pub const SLOTS_PER_EPOCH: u64 = 32;
pub const DOMAIN_SYNC_COMMITTEE: DomainType = [0x07, 0x00, 0x00, 0x00];

pub const FINALIZED_ROOT_INDEX: u32 = 105;
pub const NEXT_SYNC_COMMITTEE_INDEX: u32 = 55;
pub const FINALITY_TREE_DEPTH: u32 = floorlog2(FINALIZED_ROOT_INDEX);
pub const FINALITY_TREE_INDEX: u32 = get_subtree_index(FINALIZED_ROOT_INDEX);
pub const SYNC_COMMITTEE_TREE_DEPTH: u32 = floorlog2(NEXT_SYNC_COMMITTEE_INDEX);
pub const SYNC_COMMITTEE_TREE_INDEX: u32 = get_subtree_index(NEXT_SYNC_COMMITTEE_INDEX);

pub const BEACON_BLOCK_BODY_TREE_DEPTH: usize = 4;
pub const L1_BEACON_BLOCK_BODY_TREE_EXECUTION_PAYLOAD_INDEX: usize = 9;
pub const L2_EXECUTION_PAYLOAD_TREE_EXECUTION_BLOCK_INDEX: usize = 12;
pub const L1_BEACON_BLOCK_BODY_PROOF_SIZE: usize = 4;
pub const L2_EXECUTION_PAYLOAD_PROOF_SIZE: usize = 4;
pub const EXECUTION_PROOF_SIZE: usize =
    L1_BEACON_BLOCK_BODY_PROOF_SIZE + L2_EXECUTION_PAYLOAD_PROOF_SIZE;

#[derive(PartialEq, BorshSerialize, BorshDeserialize)]
pub enum Network {
    Mainnet,
    Kiln,
    Goerli,
}

impl FromStr for Network {
    type Err = String;
    fn from_str(input: &str) -> Result<Network, Self::Err> {
        match input {
            "mainnet" => Ok(Network::Mainnet),
            "kiln" => Ok(Network::Kiln),
            "goerli" => Ok(Network::Goerli),
            _ => Err(format!("Unknown network {}", input)),
        }
    }
}

pub struct NetworkConfig {
    pub genesis_validators_root: [u8; 32],
    pub bellatrix_fork_version: ForkVersion,
    pub bellatrix_fork_epoch: u64,
    pub capella_fork_version: ForkVersion,
    pub capella_fork_epoch: u64,
}

impl NetworkConfig {
    pub fn new(network: &Network) -> Self {
        match network {
            Network::Mainnet => Self {
                genesis_validators_root: [
                    0x4b, 0x36, 0x3d, 0xb9, 0x4e, 0x28, 0x61, 0x20, 0xd7, 0x6e, 0xb9, 0x05, 0x34,
                    0x0f, 0xdd, 0x4e, 0x54, 0xbf, 0xe9, 0xf0, 0x6b, 0xf3, 0x3f, 0xf6, 0xcf, 0x5a,
                    0xd2, 0x7f, 0x51, 0x1b, 0xfe, 0x95,
                ],
                bellatrix_fork_version: [0x02, 0x00, 0x00, 0x00],
                bellatrix_fork_epoch: 144896,
                capella_fork_version: [0; 4],
                capella_fork_epoch: 18446744073709551615,
            },
            Network::Goerli => Self {
                genesis_validators_root: [
                    0x04, 0x3d, 0xb0, 0xd9, 0xa8, 0x38, 0x13, 0x55, 0x1e, 0xe2, 0xf3, 0x34, 0x50,
                    0xd2, 0x37, 0x97, 0x75, 0x7d, 0x43, 0x09, 0x11, 0xa9, 0x32, 0x05, 0x30, 0xad,
                    0x8a, 0x0e, 0xab, 0xc4, 0x3e, 0xfb,
                ],
                bellatrix_fork_version: [0x02, 0x00, 0x10, 0x20],
                bellatrix_fork_epoch: 112260,
                capella_fork_version: [0x03, 0x00, 0x10, 0x20],
                capella_fork_epoch: 162304,
            },
            Network::Kiln => Self {
                genesis_validators_root: [
                    0x99, 0xb0, 0x9f, 0xcd, 0x43, 0xe5, 0x90, 0x52, 0x36, 0xc3, 0x70, 0xf1, 0x84,
                    0x05, 0x6b, 0xec, 0x6e, 0x66, 0x38, 0xcf, 0xc3, 0x1a, 0x32, 0x3b, 0x30, 0x4f,
                    0xc4, 0xaa, 0x78, 0x9c, 0xb4, 0xad,
                ],
                bellatrix_fork_version: [0x70, 0x00, 0x00, 0x71],
                bellatrix_fork_epoch: 150,
                capella_fork_version: [0; 4],
                capella_fork_epoch: 18446744073709551615,
            },
        }
    }

    pub fn compute_fork_version(&self, epoch: Epoch) -> Option<ForkVersion> {
        if epoch >= self.capella_fork_epoch {
            return Some(self.capella_fork_version);
        }

        if epoch >= self.bellatrix_fork_epoch {
            return Some(self.bellatrix_fork_version);
        }

        None
    }

    pub fn compute_fork_version_by_slot(&self, slot: Slot) -> Option<ForkVersion> {
        self.compute_fork_version(compute_epoch_at_slot(slot))
    }
}

pub const fn compute_epoch_at_slot(slot: Slot) -> u64 {
    slot / SLOTS_PER_EPOCH
}

pub const fn compute_sync_committee_period(slot: Slot) -> u64 {
    compute_epoch_at_slot(slot) / EPOCHS_PER_SYNC_COMMITTEE_PERIOD
}

// Compute floor of log2 of a u32.
pub const fn floorlog2(x: u32) -> u32 {
    if x == 0 {
        return 0;
    }
    31 - x.leading_zeros()
}

pub const fn get_subtree_index(generalized_index: u32) -> u32 {
    generalized_index % 2u32.pow(floorlog2(generalized_index))
}

pub fn compute_domain(
    domain_constant: DomainType,
    fork_version: ForkVersion,
    genesis_validators_root: H256,
) -> H256 {
    let fork_data_root = ForkData {
        current_version: fork_version,
        genesis_validators_root,
    }
    .tree_hash_root();

    let mut domain = [0; 32];
    domain[0..4].copy_from_slice(&domain_constant);
    domain[4..].copy_from_slice(
        fork_data_root
            .as_bytes()
            .get(..28)
            .expect("fork has is 32 bytes so first 28 bytes should exist"),
    );

    H256::from(domain)
}

pub fn compute_signing_root(object_root: H256, domain: H256) -> H256 {
    eth_types::H256(
        SigningData {
            object_root,
            domain,
        }
        .tree_hash_root(),
    )
}

pub fn get_participant_pubkeys(
    public_keys: &[PublicKeyBytes],
    sync_committee_bits: &BitVec<u8, Lsb0>,
) -> Vec<PublicKeyBytes> {
    let mut result: Vec<PublicKeyBytes> = vec![];
    for (idx, bit) in sync_committee_bits.iter().by_vals().enumerate() {
        if bit {
            result.push(public_keys[idx].clone());
        }
    }
    result
}

pub fn convert_branch(branch: &[H256]) -> Vec<ethereum_types::H256> {
    branch.iter().map(|x| x.0).collect()
}

pub fn validate_beacon_block_header_update(header_update: &HeaderUpdate) -> bool {
    let branch = convert_branch(&header_update.execution_hash_branch);
    if branch.len() != EXECUTION_PROOF_SIZE {
        return false;
    }

    let l2_proof = &branch[0..L2_EXECUTION_PAYLOAD_PROOF_SIZE];
    let l1_proof = &branch[L2_EXECUTION_PAYLOAD_PROOF_SIZE..EXECUTION_PROOF_SIZE];
    let execution_payload_hash = merkle_proof::merkle_root_from_branch(
        header_update.execution_block_hash.0,
        l2_proof,
        L2_EXECUTION_PAYLOAD_PROOF_SIZE,
        L2_EXECUTION_PAYLOAD_TREE_EXECUTION_BLOCK_INDEX,
    );
    merkle_proof::verify_merkle_proof(
        execution_payload_hash,
        l1_proof,
        BEACON_BLOCK_BODY_TREE_DEPTH,
        L1_BEACON_BLOCK_BODY_TREE_EXECUTION_PAYLOAD_INDEX,
        header_update.beacon_header.body_root.0,
    )
}

pub fn calculate_min_storage_balance_for_submitter(
    max_submitted_blocks_by_account: u32,
) -> Balance {
    const STORAGE_BYTES_PER_BLOCK: u128 = 105; // prefix: 3B + key: 32B + HeaderInfo 70B
    const STORAGE_BYTES_PER_ACCOUNT: u128 = 39; // prefix: 3B + account_id: 32B + counter 4B
    let storage_bytes_per_account = (STORAGE_BYTES_PER_BLOCK
        * max_submitted_blocks_by_account as u128)
        + STORAGE_BYTES_PER_ACCOUNT;
    storage_bytes_per_account * env::STORAGE_PRICE_PER_BYTE
}
