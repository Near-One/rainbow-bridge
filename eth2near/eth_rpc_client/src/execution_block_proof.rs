use crate::beacon_block_body_merkle_tree::{BeaconBlockBodyMerkleTree, ExecutionPayloadMerkleTree};
use crate::errors::{MerkleTreeError, MissExecutionPayload};
use ethereum_hashing;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use types::Hash256;
use types::{BeaconBlockBody, ExecutionPayload, ForkName, MainnetEthSpec};

/// `ExecutionBlockProof` contains a `block_hash` (execution block) and
/// a proof of its inclusion in the `BeaconBlockBody` tree hash.
/// The `block_hash` is the 12th field in execution_payload, which is the 9th field in `BeaconBlockBody`
/// The first 4 elements in proof correspondent to the proof of inclusion of `block_hash` in
/// Merkle tree built for `ExecutionPayload`.
/// The last 4 elements of the proof of `ExecutionPayload` in the Merkle tree are built
/// on high-level `BeaconBlockBody` fields.
/// The proof starts from the leaf.
pub struct ExecutionBlockProof {
    block_hash: Hash256,
    proof: Vec<Hash256>,
}

impl ExecutionBlockProof {
    pub const L1_BEACON_BLOCK_BODY_TREE_EXECUTION_PAYLOAD_INDEX: usize = 9;
    pub const L1_BEACON_BLOCK_BODY_PROOF_SIZE: usize = 4;
    pub const L2_EXECUTION_PAYLOAD_TREE_EXECUTION_BLOCK_INDEX: usize = 12;

    pub fn construct_from_raw_data(block_hash: Hash256, proof: Vec<Hash256>) -> Self {
        Self { block_hash, proof }
    }

    pub fn construct_from_beacon_block_body(
        beacon_block_body: &BeaconBlockBody<MainnetEthSpec>,
    ) -> Result<Self, Box<dyn Error>> {
        let l2_execution_payload_proof_size = match beacon_block_body.to_ref().fork_name() {
            ForkName::Base | ForkName::Altair | ForkName::Capella => 4,
            _ => 5,
        };

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
                l2_execution_payload_proof_size,
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

    pub fn get_proof(&self) -> Vec<Hash256> {
        self.proof.clone()
    }

    pub fn get_execution_block_hash(&self) -> Hash256 {
        self.block_hash
    }

    fn merkle_root_from_branch(
        leaf: Hash256,
        branch: &[Hash256],
        depth: usize,
        index: usize,
    ) -> Result<Hash256, IncorrectBranchLength> {
        if branch.len() != depth {
            return Err(IncorrectBranchLength);
        }

        let mut merkle_root = leaf.as_slice().to_vec();

        for (i, leaf) in branch.iter().enumerate().take(depth) {
            let ith_bit = (index >> i) & 0x01;
            if ith_bit == 1 {
                merkle_root =
                    ethereum_hashing::hash32_concat(leaf.as_slice(), &merkle_root)[..].to_vec();
            } else {
                let mut input = merkle_root;
                input.extend_from_slice(leaf.as_slice());
                merkle_root = ethereum_hashing::hash(&input);
            }
        }

        Ok(Hash256::from_slice(&merkle_root))
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
    use eth2_utility::consensus::{Network, NetworkConfig};
    use types::MainnetEthSpec;
    use types::{BeaconBlockBody, ExecutionPayload};

    const TIMEOUT_SECONDS: u64 = 30;
    const TIMEOUT_STATE_SECONDS: u64 = 1000;

    fn get_test_config() -> ConfigForTests {
        ConfigForTests::load_from_toml("config_for_tests.toml".try_into().unwrap())
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
