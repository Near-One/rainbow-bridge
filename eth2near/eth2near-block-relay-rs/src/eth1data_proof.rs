use crate::beacon_block_body_merkle_tree::BeaconBlockBodyMerkleTree;
use ethereum_types::H256;
use types::MainnetEthSpec;
use types::{BeaconBlockBody, BeaconStateError};

/// `Eth1DataProof` contains a hash for `Eth1Data` (execution block) and a proof of its inclusion in `BeaconBlockBody` tree hash.
/// The proof of `Eth1Data` in Merkle tree is built on high-level `BeaconBlockBody` fields.
/// The proof starts from the leaf.
pub struct Eth1DataProof {
    eth1data_hash: H256,
    proof: [H256; Self::BEACON_BLOCK_BODY_PROOF_SIZE],
}

impl Eth1DataProof {
    pub const BEACON_BLOCK_BODY_TREE_ETH1DATA_INDEX: usize = 1;
    pub const BEACON_BLOCK_BODY_PROOF_SIZE: usize =
        BeaconBlockBodyMerkleTree::BEACON_BLOCK_BODY_TREE_DEPTH;

    pub fn construct_from_raw_data(
        eth1data_hash: &H256,
        proof: &[H256; Self::BEACON_BLOCK_BODY_PROOF_SIZE],
    ) -> Self {
        Self {
            eth1data_hash: *eth1data_hash,
            proof: *proof,
        }
    }

    pub fn construct_from_beacon_block_body_merkle_tree(
        merkle_tree: &BeaconBlockBodyMerkleTree,
    ) -> Result<Self, BeaconStateError> {
        let eth1_proof = merkle_tree.0.generate_proof(
            Self::BEACON_BLOCK_BODY_TREE_ETH1DATA_INDEX,
            Self::BEACON_BLOCK_BODY_PROOF_SIZE,
        );

        Ok(Self {
            eth1data_hash: eth1_proof.0,
            proof: eth1_proof.1.as_slice().try_into().map_err(|_| {
                // This kind of error could clearly represent if the generated proof was of
                // different size than the expected one, which will have a defined num of leaves
                BeaconStateError::TreeHashError(tree_hash::Error::MaximumLeavesExceeded {
                    max_leaves: Self::BEACON_BLOCK_BODY_PROOF_SIZE,
                })
            })?,
        })
    }

    pub fn construct_from_beacon_block_body(
        beacon_block_body: &BeaconBlockBody<MainnetEthSpec>,
    ) -> Result<Self, BeaconStateError> {
        Self::construct_from_beacon_block_body_merkle_tree(&BeaconBlockBodyMerkleTree::new(
            beacon_block_body,
        )?)
    }

    pub fn get_proof(&self) -> [H256; Self::BEACON_BLOCK_BODY_PROOF_SIZE] {
        self.proof
    }

    pub fn get_eth1data_hash(&self) -> H256 {
        self.eth1data_hash
    }

    pub fn verify_proof_for_hash(&self, beacon_block_body_hash: &H256) -> bool {
        merkle_proof::verify_merkle_proof(
            self.eth1data_hash,
            &self.proof,
            BeaconBlockBodyMerkleTree::BEACON_BLOCK_BODY_TREE_DEPTH,
            Self::BEACON_BLOCK_BODY_TREE_ETH1DATA_INDEX,
            *beacon_block_body_hash,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::read_json_file_from_data_dir;
    use types::BeaconBlockBody;
    use types::MainnetEthSpec;

    const TEST_BEACON_BLOCK_ID: u32 = 741888;

    #[test]
    fn test_beacon_block_body_root_verification() {
        let beacon_block_body_json_str =
            read_json_file_from_data_dir("beacon_block_body_kiln_slot_741888.json");

        let beacon_block_body: BeaconBlockBody<MainnetEthSpec> =
            serde_json::from_str(&beacon_block_body_json_str).unwrap();

        let beacon_block_body_merkle_tree =
            crate::beacon_block_body_merkle_tree::BeaconBlockBodyMerkleTree::new(
                &beacon_block_body,
            )
            .unwrap();
        assert_eq!(
            format!("{:?}", beacon_block_body_merkle_tree.0.hash()),
            "0xd7f1c80baaceb9a1d3301e4f740fe8b5de9970153dc2ab254a4be39fe054addc"
        );

        use tree_hash::TreeHash;
        let eth1data_proof =
            crate::eth1data_proof::Eth1DataProof::construct_from_beacon_block_body_merkle_tree(
                &beacon_block_body_merkle_tree,
            )
            .unwrap();
        assert_eq!(
            beacon_block_body.eth1_data().tree_hash_root(),
            eth1data_proof.get_eth1data_hash()
        );
        assert!(eth1data_proof.verify_proof_for_hash(&beacon_block_body_merkle_tree.0.hash()));

        let eth1data_proof_copy = crate::eth1data_proof::Eth1DataProof::construct_from_raw_data(
            &eth1data_proof.get_eth1data_hash(),
            &eth1data_proof.get_proof(),
        );
        assert!(eth1data_proof_copy.verify_proof_for_hash(&beacon_block_body_merkle_tree.0.hash()));

        let eth1data_proof_from_beacon_body =
            crate::eth1data_proof::Eth1DataProof::construct_from_beacon_block_body(
                &beacon_block_body,
            )
            .unwrap();
        assert_eq!(
            eth1data_proof.get_proof(),
            eth1data_proof_from_beacon_body.get_proof()
        );
    }

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_beacon_block_body_root_matches_body_root_in_header() {
        let beacon_block_body = aw!(crate::beacon_rpc_client::BeaconRPCClient::default()
            .get_beacon_block_body_for_block_id(&TEST_BEACON_BLOCK_ID.to_string()))
        .unwrap();
        let beacon_block_header = aw!(crate::beacon_rpc_client::BeaconRPCClient::default()
            .get_beacon_block_header_for_block_id(&TEST_BEACON_BLOCK_ID.to_string()))
        .unwrap();

        let beacon_block_body_merkle_tree =
            crate::beacon_block_body_merkle_tree::BeaconBlockBodyMerkleTree::new(
                &beacon_block_body,
            )
            .unwrap()
            .0;
        assert_eq!(
            beacon_block_body_merkle_tree.hash(),
            beacon_block_header.body_root
        );
    }
}
