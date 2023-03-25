use ethereum_types::H256;
use merkle_proof::MerkleTree;
use tree_hash::TreeHash;
use types::{BeaconBlockBody, BeaconState, ExecutionPayload, MainnetEthSpec};

/// `BeaconBlockBodyMerkleTree` is built on the `BeaconBlockBody` data structure,
/// where the leaves of the Merkle Tree are the hashes of the
/// high-level fields of the `BeaconBlockBody`.
/// The hashes of each element are produced by using `ssz` serialization.
pub struct BeaconBlockBodyMerkleTree(pub MerkleTree);

impl BeaconBlockBodyMerkleTree {
    pub const BEACON_BLOCK_BODY_TREE_NUM_LEAVES: usize = 11;
    pub const BEACON_BLOCK_BODY_TREE_DEPTH: usize = 4;

    pub fn new(beacon_block_body: &BeaconBlockBody<MainnetEthSpec>) -> Self {
        let leaves: [H256; Self::BEACON_BLOCK_BODY_TREE_NUM_LEAVES] = [
            beacon_block_body.randao_reveal().tree_hash_root(),
            beacon_block_body.eth1_data().tree_hash_root(),
            beacon_block_body.graffiti().tree_hash_root(),
            beacon_block_body.proposer_slashings().tree_hash_root(),
            beacon_block_body.attester_slashings().tree_hash_root(),
            beacon_block_body.attestations().tree_hash_root(),
            beacon_block_body.deposits().tree_hash_root(),
            beacon_block_body.voluntary_exits().tree_hash_root(),
            if let Ok(sync_aggregate) = beacon_block_body.sync_aggregate() {
                sync_aggregate.tree_hash_root()
            } else {
                H256::zero()
            },
            if let Ok(execution_payload) = beacon_block_body.execution_payload() {
                execution_payload.tree_hash_root()
            } else {
                H256::zero()
            },
            if let Ok(bls_to_execution_changes) = beacon_block_body.bls_to_execution_changes() {
                bls_to_execution_changes.tree_hash_root()
            } else {
                H256::zero()
            },
        ];

        Self(MerkleTree::create(
            &leaves,
            Self::BEACON_BLOCK_BODY_TREE_DEPTH,
        ))
    }
}

/// `ExecutionPayloadMerkleTree` is a built on the `ExecutionPayload` data structure,
/// where the leaves of the Merkle Tree are the hashes of the
/// high-level fields of the `ExecutionPayload`.
/// The hashes of each element are produced by using `ssz` serialization.
/// `ExecutionPayload` is one of the field in BeaconBlockBody.
/// The hash of the root of `ExecutionPlayloadMerkleTree` is the 9th leaf in BeaconBlockBody Merkle Tree.
pub struct ExecutionPayloadMerkleTree(pub MerkleTree);

impl ExecutionPayloadMerkleTree {
    pub const TREE_NUM_LEAVES: usize = 15;
    pub const TREE_DEPTH: usize = 4;

    pub fn new(execution_payload: &ExecutionPayload<MainnetEthSpec>) -> Self {
        let leaves: [H256; Self::TREE_NUM_LEAVES] = [
            execution_payload.parent_hash().tree_hash_root(),
            execution_payload.fee_recipient().tree_hash_root(),
            execution_payload.state_root().tree_hash_root(),
            execution_payload.receipts_root().tree_hash_root(),
            execution_payload.logs_bloom().tree_hash_root(),
            execution_payload.prev_randao().tree_hash_root(),
            execution_payload.block_number().tree_hash_root(),
            execution_payload.gas_limit().tree_hash_root(),
            execution_payload.gas_used().tree_hash_root(),
            execution_payload.timestamp().tree_hash_root(),
            execution_payload.extra_data().tree_hash_root(),
            execution_payload.base_fee_per_gas().tree_hash_root(),
            execution_payload.block_hash().tree_hash_root(),
            execution_payload.transactions().tree_hash_root(),
            if let Ok(withdrawals) = execution_payload.withdrawals() {
                withdrawals.tree_hash_root()
            } else {
                H256::zero()
            },
        ];

        Self(MerkleTree::create(&leaves, Self::TREE_DEPTH))
    }
}

pub struct BeaconStateMerkleTree(pub MerkleTree);

impl BeaconStateMerkleTree {
    pub const TREE_NUM_LEAVES: usize = 28;
    pub const TREE_DEPTH: usize = 5;

    pub fn new(beacon_state: &BeaconState<MainnetEthSpec>) -> Self {
        let leaves: [H256; Self::TREE_NUM_LEAVES] = [
            beacon_state.genesis_time().tree_hash_root(),
            beacon_state.genesis_validators_root().tree_hash_root(),
            beacon_state.slot().tree_hash_root(),
            beacon_state.fork().tree_hash_root(),
            beacon_state.latest_block_header().tree_hash_root(),
            beacon_state.block_roots().tree_hash_root(),
            beacon_state.state_roots().tree_hash_root(),
            beacon_state.historical_roots().tree_hash_root(),
            beacon_state.eth1_data().tree_hash_root(),
            beacon_state.eth1_data_votes().tree_hash_root(),
            beacon_state.eth1_deposit_index().tree_hash_root(),
            beacon_state.validators().tree_hash_root(),
            beacon_state.balances().tree_hash_root(),
            beacon_state.randao_mixes().tree_hash_root(),
            beacon_state.slashings().tree_hash_root(),
            if let Ok(previous_epoch_participation) = beacon_state.previous_epoch_participation() {
                previous_epoch_participation.tree_hash_root()
            } else {
                H256::zero()
            },
            if let Ok(current_epoch_participation) = beacon_state.current_epoch_participation() {
                current_epoch_participation.tree_hash_root()
            } else {
                H256::zero()
            },
            beacon_state.justification_bits().tree_hash_root(),
            beacon_state
                .previous_justified_checkpoint()
                .tree_hash_root(),
            beacon_state.current_justified_checkpoint().tree_hash_root(),
            beacon_state.finalized_checkpoint().tree_hash_root(),
            if let Ok(inactivity_scores) = beacon_state.inactivity_scores() {
                inactivity_scores.tree_hash_root()
            } else {
                H256::zero()
            },
            if let Ok(current_sync_committee) = beacon_state.current_sync_committee() {
                current_sync_committee.tree_hash_root()
            } else {
                H256::zero()
            },
            if let Ok(next_sync_committee) = beacon_state.next_sync_committee() {
                next_sync_committee.tree_hash_root()
            } else {
                H256::zero()
            },
            if let Ok(latest_execution_payload_header) =
                beacon_state.latest_execution_payload_header()
            {
                latest_execution_payload_header.tree_hash_root()
            } else {
                H256::zero()
            },
            if let Ok(next_withdrawal_index) = beacon_state.next_withdrawal_index() {
                next_withdrawal_index.tree_hash_root()
            } else {
                H256::zero()
            },
            if let Ok(next_withdrawal_validator_index) =
                beacon_state.next_withdrawal_validator_index()
            {
                next_withdrawal_validator_index.tree_hash_root()
            } else {
                H256::zero()
            },
            if let Ok(historical_summaries) = beacon_state.historical_summaries() {
                historical_summaries.tree_hash_root()
            } else {
                H256::zero()
            },
        ];

        Self(MerkleTree::create(&leaves, Self::TREE_DEPTH))
    }
}

#[cfg(test)]
mod tests {
    use crate::beacon_block_body_merkle_tree::{
        BeaconBlockBodyMerkleTree, ExecutionPayloadMerkleTree,
    };
    use crate::utils::read_json_file_from_data_dir;
    use tree_hash::TreeHash;
    use types::MainnetEthSpec;
    use types::{BeaconBlockBody, ExecutionPayload};

    #[test]
    fn test_body_root() {
        let json_str = read_json_file_from_data_dir("beacon_block_body_goerli_slot_5262172.json");
        let beacon_block_body: BeaconBlockBody<MainnetEthSpec> =
            serde_json::from_str(&json_str).unwrap();

        let merkle_tree = BeaconBlockBodyMerkleTree::new(&beacon_block_body);
        assert_eq!(
            format!("{:?}", merkle_tree.0.hash()),
            "0x5f3a9eda5c6d2f5c30e4ad2f9c5221334deec7ea2e3ba2b21b78cf10c7f9b1fe"
        );
    }

    #[test]
    fn test_execution_payload_merkle_tree() {
        const EXECUTION_PAYLOAD_INDEX: usize = 9;

        let json_str = read_json_file_from_data_dir("beacon_block_body_goerli_slot_5262172.json");
        let beacon_block_body: BeaconBlockBody<MainnetEthSpec> =
            serde_json::from_str(&json_str).unwrap();
        let beacon_block_body_merkle_tree = BeaconBlockBodyMerkleTree::new(&beacon_block_body);
        let execution_payload: ExecutionPayload<MainnetEthSpec> =
            beacon_block_body.execution_payload().unwrap().into();
        let execution_payload_merkle_tree = ExecutionPayloadMerkleTree::new(&execution_payload);

        assert_eq!(
            execution_payload.tree_hash_root(),
            execution_payload_merkle_tree.0.hash()
        );

        let execution_payload_proof = beacon_block_body_merkle_tree.0.generate_proof(
            EXECUTION_PAYLOAD_INDEX,
            BeaconBlockBodyMerkleTree::BEACON_BLOCK_BODY_TREE_DEPTH,
        );
        assert_eq!(
            execution_payload_proof.clone().unwrap().0,
            execution_payload_merkle_tree.0.hash()
        );
        assert!(merkle_proof::verify_merkle_proof(
            execution_payload_merkle_tree.0.hash(),
            &execution_payload_proof.unwrap().1,
            BeaconBlockBodyMerkleTree::BEACON_BLOCK_BODY_TREE_DEPTH,
            EXECUTION_PAYLOAD_INDEX,
            beacon_block_body_merkle_tree.0.hash()
        ));
    }
}
