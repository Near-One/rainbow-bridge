use ethereum_types::H256;
use merkle_proof::MerkleTree;
use tree_hash::TreeHash;
use types::MainnetEthSpec;
use types::{BeaconBlockBody, BeaconStateError};

/// `BeaconBlockBodyMerkleTree` is built on the `BeaconBlockBody` data structure,
/// where the leaves of the Merkle Tree are the hashes of the
/// high-level fields of the `BeaconBlockBody`.
/// The hashes of each element are produced by using `ssz` serialization.
pub struct BeaconBlockBodyMerkleTree(pub MerkleTree);

impl BeaconBlockBodyMerkleTree {
    pub const BEACON_BLOCK_BODY_TREE_NUM_LEAVES: usize = 10;
    pub const BEACON_BLOCK_BODY_TREE_DEPTH: usize = 4;

    pub fn new(
        beacon_block_body: &BeaconBlockBody<MainnetEthSpec>,
    ) -> Result<Self, BeaconStateError> {
        let leaves: [H256; Self::BEACON_BLOCK_BODY_TREE_NUM_LEAVES] = [
            beacon_block_body.randao_reveal().tree_hash_root(),
            beacon_block_body.eth1_data().tree_hash_root(),
            beacon_block_body.graffiti().tree_hash_root(),
            beacon_block_body.proposer_slashings().tree_hash_root(),
            beacon_block_body.attester_slashings().tree_hash_root(),
            beacon_block_body.attestations().tree_hash_root(),
            beacon_block_body.deposits().tree_hash_root(),
            beacon_block_body.voluntary_exits().tree_hash_root(),
            beacon_block_body.sync_aggregate()?.tree_hash_root(),
            beacon_block_body.execution_payload()?.tree_hash_root(),
        ];

        Ok(Self(MerkleTree::create(
            &leaves,
            Self::BEACON_BLOCK_BODY_TREE_DEPTH,
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::beacon_block_body_merkle_tree::BeaconBlockBodyMerkleTree;
    use crate::test_utils::read_json_file_from_data_dir;
    use types::BeaconBlockBody;
    use types::MainnetEthSpec;

    #[test]
    fn test_body_root() {
        let json_str = read_json_file_from_data_dir("beacon_block_body_kiln_slot_741888.json");
        let beacon_block_body: BeaconBlockBody<MainnetEthSpec> =
            serde_json::from_str(&json_str).unwrap();

        let merkle_tree = BeaconBlockBodyMerkleTree::new(&beacon_block_body).unwrap();
        assert_eq!(
            format!("{:?}", merkle_tree.0.hash()),
            "0xd7f1c80baaceb9a1d3301e4f740fe8b5de9970153dc2ab254a4be39fe054addc"
        );
    }
}
