use crate::beacon_block_body_merkle_tree::{BeaconBlockBodyMerkleTree, ExecutionPayloadMerkleTree};
use crate::errors::{MerkleTreeError, MissExecutionPayload};
use eth2_hashing;
use ethereum_types::H256;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use types::{BeaconBlockBody, ExecutionPayload, MainnetEthSpec};

/// `ExecutionBlockProof` contains a `block_hash` (execution block) and
/// a proof of its inclusion in the `BeaconBlockBody` tree hash.
/// The `block_hash` is the 12th field in execution_payload, which is the 9th field in `BeaconBlockBody`
/// The first 4 elements in proof correspondent to the proof of inclusion of `block_hash` in
/// Merkle tree built for `ExecutionPayload`.
/// The last 4 elements of the proof of `ExecutionPayload` in the Merkle tree are built
/// on high-level `BeaconBlockBody` fields.
/// The proof starts from the leaf.
pub struct ExecutionBlockProof {
    block_hash: H256,
    proof: [H256; Self::PROOF_SIZE],
}

impl ExecutionBlockProof {
    pub const L1_BEACON_BLOCK_BODY_TREE_EXECUTION_PAYLOAD_INDEX: usize = 9;
    pub const L2_EXECUTION_PAYLOAD_TREE_EXECUTION_BLOCK_INDEX: usize = 12;

    pub const L1_BEACON_BLOCK_BODY_PROOF_SIZE: usize =
        BeaconBlockBodyMerkleTree::BEACON_BLOCK_BODY_TREE_DEPTH;
    pub const L2_EXECUTION_PAYLOAD_PROOF_SIZE: usize = ExecutionPayloadMerkleTree::TREE_DEPTH;
    pub const PROOF_SIZE: usize =
        Self::L1_BEACON_BLOCK_BODY_PROOF_SIZE + Self::L2_EXECUTION_PAYLOAD_PROOF_SIZE;

    pub fn construct_from_raw_data(block_hash: &H256, proof: &[H256; Self::PROOF_SIZE]) -> Self {
        Self {
            block_hash: *block_hash,
            proof: *proof,
        }
    }

    pub fn construct_from_beacon_block_body(
        beacon_block_body: &BeaconBlockBody<MainnetEthSpec>,
    ) -> Result<Self, Box<dyn Error>> {
        let beacon_block_merkle_tree = &BeaconBlockBodyMerkleTree::new(beacon_block_body);

        let execution_payload_merkle_tree = &ExecutionPayloadMerkleTree::new(
            &beacon_block_body
                .execution_payload()
                .map_err(|_| MissExecutionPayload)?
                .into(),
        );

        let l1_execution_payload_proof = beacon_block_merkle_tree
            .0
            .generate_proof(
                Self::L1_BEACON_BLOCK_BODY_TREE_EXECUTION_PAYLOAD_INDEX,
                Self::L1_BEACON_BLOCK_BODY_PROOF_SIZE,
            )
            .map_err(|err| MerkleTreeError(err))?
            .1;
        let mut block_proof = execution_payload_merkle_tree
            .0
            .generate_proof(
                Self::L2_EXECUTION_PAYLOAD_TREE_EXECUTION_BLOCK_INDEX,
                Self::L2_EXECUTION_PAYLOAD_PROOF_SIZE,
            )
            .map_err(|err| MerkleTreeError(err))?
            .1;
        block_proof.extend(&l1_execution_payload_proof);

        let execution_payload: ExecutionPayload<MainnetEthSpec> = beacon_block_body
            .execution_payload()
            .map_err(|_| MissExecutionPayload)?
            .into();
        Ok(Self {
            block_hash: execution_payload.block_hash().into_root(),
            proof: block_proof.as_slice().try_into()?,
        })
    }

    pub fn get_proof(&self) -> [H256; Self::PROOF_SIZE] {
        self.proof
    }

    pub fn get_execution_block_hash(&self) -> H256 {
        self.block_hash
    }

    pub fn verify_proof_for_hash(
        &self,
        beacon_block_body_hash: &H256,
    ) -> Result<bool, IncorrectBranchLength> {
        let l2_proof: &[H256] = &self.proof[0..Self::L2_EXECUTION_PAYLOAD_PROOF_SIZE];
        let l1_proof: &[H256] =
            &self.proof[Self::L2_EXECUTION_PAYLOAD_PROOF_SIZE..Self::PROOF_SIZE];
        let execution_payload_hash = Self::merkle_root_from_branch(
            self.block_hash,
            l2_proof,
            Self::L2_EXECUTION_PAYLOAD_PROOF_SIZE,
            Self::L2_EXECUTION_PAYLOAD_TREE_EXECUTION_BLOCK_INDEX,
        )?;

        Ok(merkle_proof::verify_merkle_proof(
            execution_payload_hash,
            l1_proof,
            BeaconBlockBodyMerkleTree::BEACON_BLOCK_BODY_TREE_DEPTH,
            Self::L1_BEACON_BLOCK_BODY_TREE_EXECUTION_PAYLOAD_INDEX,
            *beacon_block_body_hash,
        ))
    }

    fn merkle_root_from_branch(
        leaf: H256,
        branch: &[H256],
        depth: usize,
        index: usize,
    ) -> Result<H256, IncorrectBranchLength> {
        if branch.len() != depth {
            return Err(IncorrectBranchLength);
        }

        let mut merkle_root = leaf.as_bytes().to_vec();

        for (i, leaf) in branch.iter().enumerate().take(depth) {
            let ith_bit = (index >> i) & 0x01;
            if ith_bit == 1 {
                merkle_root =
                    eth2_hashing::hash32_concat(leaf.as_bytes(), &merkle_root)[..].to_vec();
            } else {
                let mut input = merkle_root;
                input.extend_from_slice(leaf.as_bytes());
                merkle_root = eth2_hashing::hash(&input);
            }
        }

        Ok(H256::from_slice(&merkle_root))
    }
}

#[derive(Debug)]
pub struct IncorrectBranchLength;

impl Display for IncorrectBranchLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error on getting merkle root from branch. Proof length should equal depth"
        )
    }
}

impl Error for IncorrectBranchLength {}

#[cfg(test)]
mod tests {
    use crate::config_for_tests::ConfigForTests;
    use crate::utils::read_json_file_from_data_dir;
    use types::MainnetEthSpec;
    use types::{BeaconBlockBody, ExecutionPayload};

    const TIMEOUT_SECONDS: u64 = 30;
    const TIMEOUT_STATE_SECONDS: u64 = 1000;

    fn get_test_config() -> ConfigForTests {
        ConfigForTests::load_from_toml("config_for_tests.toml".try_into().unwrap())
    }

    #[test]
    fn test_beacon_block_body_root_verification() {
        let beacon_block_body_json_str =
            read_json_file_from_data_dir("beacon_block_body_goerli_slot_5262172.json");

        let beacon_block_body: BeaconBlockBody<MainnetEthSpec> =
            serde_json::from_str(&beacon_block_body_json_str).unwrap();

        let beacon_block_body_merkle_tree =
            crate::beacon_block_body_merkle_tree::BeaconBlockBodyMerkleTree::new(
                &beacon_block_body,
            );

        assert_eq!(
            format!("{:?}", beacon_block_body_merkle_tree.0.hash()),
            "0x5f3a9eda5c6d2f5c30e4ad2f9c5221334deec7ea2e3ba2b21b78cf10c7f9b1fe"
        );

        let execution_block_proof =
            crate::execution_block_proof::ExecutionBlockProof::construct_from_beacon_block_body(
                &beacon_block_body,
            )
            .unwrap();

        let execution_payload: ExecutionPayload<MainnetEthSpec> =
            beacon_block_body.execution_payload().unwrap().into();
        assert_eq!(
            execution_payload.block_hash(),
            types::ExecutionBlockHash::from_root(execution_block_proof.get_execution_block_hash())
        );

        assert!(execution_block_proof
            .verify_proof_for_hash(&beacon_block_body_merkle_tree.0.hash())
            .unwrap());

        let execution_block_proof_copy =
            crate::execution_block_proof::ExecutionBlockProof::construct_from_raw_data(
                &execution_block_proof.get_execution_block_hash(),
                &execution_block_proof.get_proof(),
            );
        assert!(execution_block_proof_copy
            .verify_proof_for_hash(&beacon_block_body_merkle_tree.0.hash())
            .unwrap());
    }

    #[test]
    fn test_beacon_block_body_root_matches_body_root_in_header() {
        let config = get_test_config();

        let beacon_rpc_client = crate::beacon_rpc_client::BeaconRPCClient::new(
            &config.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
            None,
        );

        let beacon_block_body = beacon_rpc_client
            .get_beacon_block_body_for_block_id(&format!("{}", config.first_slot))
            .unwrap();
        let beacon_block_header = beacon_rpc_client
            .get_beacon_block_header_for_block_id(&format!("{}", config.first_slot))
            .unwrap();

        let beacon_block_body_merkle_tree =
            crate::beacon_block_body_merkle_tree::BeaconBlockBodyMerkleTree::new(
                &beacon_block_body,
            )
            .0;
        assert_eq!(
            beacon_block_body_merkle_tree.hash(),
            beacon_block_header.body_root
        );
    }
}
