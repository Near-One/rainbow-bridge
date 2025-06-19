use std::str::FromStr;

use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use eth_types::eth2::*;
use eth_types::H256;
use near_sdk::near;
use tree_hash::TreeHash;

pub const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: u64 = 256;
pub const MIN_SYNC_COMMITTEE_PARTICIPANTS: u64 = 1;
pub const SLOTS_PER_EPOCH: u64 = 32;
pub const DOMAIN_SYNC_COMMITTEE: DomainType = [0x07, 0x00, 0x00, 0x00];

// Generalized indices for execution payload
pub const EXECUTION_PAYLOAD_GINDEX: u32 = 25; // Pre-Electra
pub const EXECUTION_PAYLOAD_GINDEX_ELECTRA: u32 = 41; // Electra and later

pub struct ProofSize {
    pub beacon_block_body_tree_depth: usize,
    pub l1_beacon_block_body_tree_execution_payload_index: usize,
    pub l2_execution_payload_tree_execution_block_index: usize,
    pub l1_beacon_block_body_proof_size: usize,
    pub l2_execution_payload_proof_size: usize,
    pub execution_proof_size: usize,
}

#[derive(Debug)]
pub struct GeneralizedIndex {
    pub finality_tree_depth: u32,
    pub finality_tree_index: u32,
    pub current_sync_committee_tree_depth: u32,
    pub current_sync_committee_tree_index: u32,
    pub sync_committee_tree_depth: u32,
    pub sync_committee_tree_index: u32,
}

#[derive(PartialEq, BorshSerialize, BorshDeserialize, BorshSchema, Clone, Copy, Debug)]
#[near(serializers=[json])]
pub enum Network {
    Mainnet,
    Goerli,
    Sepolia,
}

impl FromStr for Network {
    type Err = String;
    fn from_str(input: &str) -> Result<Network, Self::Err> {
        match input {
            "mainnet" => Ok(Network::Mainnet),
            "goerli" => Ok(Network::Goerli),
            "sepolia" => Ok(Network::Sepolia),
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
    pub deneb_fork_version: ForkVersion,
    pub deneb_fork_epoch: u64,
    pub electra_fork_version: ForkVersion,
    pub electra_fork_epoch: u64,
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
                capella_fork_version: [0x03, 0x00, 0x00, 0x00],
                capella_fork_epoch: 194048,
                deneb_fork_version: [0x04, 0x00, 0x00, 0x00],
                deneb_fork_epoch: 269568,
                electra_fork_version: [0x05, 0x00, 0x00, 0x00],
                electra_fork_epoch: 364032,
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
                deneb_fork_version: [0x04, 0x00, 0x10, 0x20],
                deneb_fork_epoch: 231680,
                electra_fork_version: [0x00, 0x00, 0x00, 0x00], // Not supported
                electra_fork_epoch: u64::MAX,                   // Not supported
            },
            Network::Sepolia => Self {
                genesis_validators_root: [
                    0xd8, 0xea, 0x17, 0x1f, 0x3c, 0x94, 0xae, 0xa2, 0x1e, 0xbc, 0x42, 0xa1, 0xed,
                    0x61, 0x05, 0x2a, 0xcf, 0x3f, 0x92, 0x09, 0xc0, 0x0e, 0x4e, 0xfb, 0xaa, 0xdd,
                    0xac, 0x09, 0xed, 0x9b, 0x80, 0x78,
                ],
                bellatrix_fork_version: [0x90, 0x00, 0x00, 0x71],
                bellatrix_fork_epoch: 100,
                capella_fork_version: [0x90, 0x00, 0x00, 0x72],
                capella_fork_epoch: 56832,
                deneb_fork_version: [0x90, 0x00, 0x00, 0x73],
                deneb_fork_epoch: 132608,
                electra_fork_version: [0x90, 0x00, 0x00, 0x74],
                electra_fork_epoch: 222464,
            },
        }
    }

    pub fn compute_fork_version(&self, epoch: Epoch) -> Option<ForkVersion> {
        if epoch >= self.electra_fork_epoch {
            return Some(self.electra_fork_version);
        }

        if epoch >= self.deneb_fork_epoch {
            return Some(self.deneb_fork_version);
        }

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

    pub fn compute_proof_size(&self, epoch: Epoch) -> ProofSize {
        if epoch >= self.deneb_fork_epoch {
            return ProofSize {
                beacon_block_body_tree_depth: 4,
                l1_beacon_block_body_tree_execution_payload_index: 9,
                l2_execution_payload_tree_execution_block_index: 12,
                l1_beacon_block_body_proof_size: 4,
                l2_execution_payload_proof_size: 5,
                execution_proof_size: 9,
            };
        }

        ProofSize {
            beacon_block_body_tree_depth: 4,
            l1_beacon_block_body_tree_execution_payload_index: 9,
            l2_execution_payload_tree_execution_block_index: 12,
            l1_beacon_block_body_proof_size: 4,
            l2_execution_payload_proof_size: 4,
            execution_proof_size: 8,
        }
    }

    pub const fn get_generalized_index_constants(&self, slot: Slot) -> GeneralizedIndex {
        // Altair
        pub const FINALIZED_ROOT_INDEX: u32 = 105;
        pub const CURRENT_SYNC_COMMITTEE_INDEX: u32 = 55;
        pub const NEXT_SYNC_COMMITTEE_INDEX: u32 = 55;

        // Electra
        pub const FINALIZED_ROOT_INDEX_ELECTRA: u32 = 169;
        pub const CURRENT_SYNC_COMMITTEE_INDEX_ELECTRA: u32 = 86;
        pub const NEXT_SYNC_COMMITTEE_INDEX_ELECTRA: u32 = 87;

        let epoch = compute_epoch_at_slot(slot);

        if epoch >= self.electra_fork_epoch {
            GeneralizedIndex {
                finality_tree_depth: floorlog2(FINALIZED_ROOT_INDEX_ELECTRA),
                finality_tree_index: get_subtree_index(FINALIZED_ROOT_INDEX_ELECTRA),
                current_sync_committee_tree_depth: floorlog2(CURRENT_SYNC_COMMITTEE_INDEX_ELECTRA),
                current_sync_committee_tree_index: get_subtree_index(
                    CURRENT_SYNC_COMMITTEE_INDEX_ELECTRA,
                ),
                sync_committee_tree_depth: floorlog2(NEXT_SYNC_COMMITTEE_INDEX_ELECTRA),
                sync_committee_tree_index: get_subtree_index(NEXT_SYNC_COMMITTEE_INDEX_ELECTRA),
            }
        } else {
            GeneralizedIndex {
                finality_tree_depth: floorlog2(FINALIZED_ROOT_INDEX),
                finality_tree_index: get_subtree_index(FINALIZED_ROOT_INDEX),
                current_sync_committee_tree_depth: floorlog2(CURRENT_SYNC_COMMITTEE_INDEX),
                current_sync_committee_tree_index: get_subtree_index(CURRENT_SYNC_COMMITTEE_INDEX),
                sync_committee_tree_depth: floorlog2(NEXT_SYNC_COMMITTEE_INDEX),
                sync_committee_tree_index: get_subtree_index(NEXT_SYNC_COMMITTEE_INDEX),
            }
        }
    }

    pub fn compute_proof_size_by_slot(&self, slot: Slot) -> ProofSize {
        self.compute_proof_size(compute_epoch_at_slot(slot))
    }

    // Fork-aware execution root computation - manual tree hashing per fork
    pub fn get_lc_execution_root(&self, header: &LightClientHeader) -> H256 {
        use tree_hash::{MerkleHasher, TreeHash};

        let epoch = compute_epoch_at_slot(header.beacon.slot);
        let execution = &header.execution;

        let mut leaves: Vec<tree_hash::Hash256> = vec![
            execution.parent_hash.tree_hash_root(),
            execution.fee_recipient.tree_hash_root(),
            execution.state_root.tree_hash_root(),
            execution.receipts_root.tree_hash_root(),
            execution.logs_bloom.tree_hash_root(),
            execution.prev_randao.tree_hash_root(),
            execution.block_number.tree_hash_root(),
            execution.gas_limit.tree_hash_root(),
            execution.gas_used.tree_hash_root(),
            execution.timestamp.tree_hash_root(),
            execution.extra_data.tree_hash_root(),
            execution.base_fee_per_gas.tree_hash_root(),
            execution.block_hash.tree_hash_root(),
            execution.transactions_root.tree_hash_root(),
        ];

        // Add withdrawals for Capella+
        if epoch >= self.capella_fork_epoch {
            leaves.push(
                execution
                    .withdrawals_root
                    .unwrap_or_default()
                    .tree_hash_root(),
            );
        }

        // Add blob fields for Deneb+
        if epoch >= self.deneb_fork_epoch {
            leaves.push(execution.blob_gas_used.unwrap_or_default().tree_hash_root());
            leaves.push(
                execution
                    .excess_blob_gas
                    .unwrap_or_default()
                    .tree_hash_root(),
            );
        }

        // Create hasher with correct number of leaves
        let mut hasher = MerkleHasher::with_leaves(leaves.len());

        // Write all leaves to hasher
        for leaf in leaves {
            hasher.write(leaf.as_slice()).unwrap();
        }

        H256(hasher.finish().unwrap().0.into())
    }

    pub fn is_valid_light_client_header(&self, header: &LightClientHeader) -> bool {
        let epoch = compute_epoch_at_slot(header.beacon.slot);
        if epoch < self.deneb_fork_epoch {
            if header.execution.blob_gas_used.is_some()
                || header.execution.excess_blob_gas.is_some()
            {
                return false;
            }
        }

        if epoch < self.capella_fork_epoch {
            panic!("Unsupported fork");
        }

        // Capella and later: verify execution payload against branch
        // The execution payload is at field index 9 in the BeaconBlockBody
        // and the beacon block body tree has depth 4 (2^4 = 16 leaves with padding)
        const EXECUTION_PAYLOAD_FIELD_INDEX: usize = 9;
        const BEACON_BLOCK_BODY_TREE_DEPTH: usize = 4;

        verify_merkle_proof(
            self.get_lc_execution_root(header),
            &header.execution_branch,
            BEACON_BLOCK_BODY_TREE_DEPTH,
            EXECUTION_PAYLOAD_FIELD_INDEX,
            header.beacon.body_root,
        )
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
            .as_slice()
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
        .tree_hash_root()
        .0
        .into(),
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

/// Verify a proof that `leaf` exists at `index` in a Merkle tree rooted at `root`.
///
/// The `branch` argument is the main component of the proof: it should be a list of internal
/// node hashes such that the root can be reconstructed (in bottom-up order).
pub fn verify_merkle_proof(
    leaf: H256,
    branch: &[H256],
    depth: usize,
    index: usize,
    root: H256,
) -> bool {
    if branch.len() == depth {
        merkle_root_from_branch(leaf, branch, depth, index) == root
    } else {
        false
    }
}

/// Compute a root hash from a leaf and a Merkle proof.
pub fn merkle_root_from_branch(leaf: H256, branch: &[H256], depth: usize, index: usize) -> H256 {
    assert_eq!(branch.len(), depth, "proof length should equal depth");

    let mut merkle_root = leaf.0.as_bytes().to_vec();

    for (i, leaf) in branch.iter().enumerate().take(depth) {
        let ith_bit = (index >> i) & 0x01;
        if ith_bit == 1 {
            merkle_root =
                ethereum_hashing::hash32_concat(leaf.0.as_bytes(), &merkle_root)[..].to_vec();
        } else {
            let mut input = merkle_root;
            input.extend_from_slice(leaf.0.as_bytes());
            merkle_root = ethereum_hashing::hash(&input);
        }
    }

    H256(ethereum_types::H256::from_slice(&merkle_root))
}
