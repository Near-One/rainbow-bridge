use crate::beacon_block_body_merkle_tree::BeaconStateMerkleTree;
use crate::beacon_rpc_client::BeaconRPCClient;
use crate::execution_block_proof::ExecutionBlockProof;
use eth_types::eth2::{
    FinalizedHeaderUpdate, HeaderUpdate, LightClientUpdate, SignatureBytes, SyncCommitteeBits,
    SyncCommitteeUpdate,
};
use std::error::Error;
use tree_hash::TreeHash;
use crate::beacon_rpc_client;

pub struct HandMadeFinalityLightClientUpdate {}

impl HandMadeFinalityLightClientUpdate {
    pub fn get_finality_light_client_update(
        beacon_rpc_client: &BeaconRPCClient,
        signature_slot: u64,
    ) -> Result<LightClientUpdate, Box<dyn Error>> {
        const BEACON_STATE_MERKLE_TREE_DEPTH: usize = 5;
        const BEACON_STATE_FINALIZED_CHECKPOINT_INDEX: usize = 20;

        let signature_beacon_body = beacon_rpc_client
            .get_beacon_block_body_for_block_id(&format!("{}", signature_slot))?;
        let sync_committe_signature = signature_beacon_body.sync_aggregate().map_err(|_| { beacon_rpc_client::MissSyncAggregationError() })?;

        let attested_slot = signature_beacon_body.attestations()[0].data.slot;

        let attested_header = beacon_rpc_client
            .get_beacon_block_header_for_block_id(&format!("{}", attested_slot))?;
        let beacon_state = beacon_rpc_client.get_beacon_state(&format!("{}", attested_slot))?;
        let finality_hash = beacon_state.finalized_checkpoint().root;
        let finality_header = beacon_rpc_client
            .get_beacon_block_header_for_block_id(&format!("{:?}", &finality_hash))?;

        let beacon_state_merkle_tree = BeaconStateMerkleTree::new(&beacon_state);
        let mut proof = beacon_state_merkle_tree.0.generate_proof(
            BEACON_STATE_FINALIZED_CHECKPOINT_INDEX,
            BEACON_STATE_MERKLE_TREE_DEPTH,
        );

        let mut finality_branch = vec![beacon_state.finalized_checkpoint().epoch.tree_hash_root()];
        finality_branch.append(&mut proof.1);

        let finalized_block_body = beacon_rpc_client
            .get_beacon_block_body_for_block_id(&format!("{:?}", &finality_hash))?;
        let finalized_block_eth1data_proof =
            ExecutionBlockProof::construct_from_beacon_block_body(&finalized_block_body)?;


        let sync_committee_bits: [u8; 64] =  match sync_committe_signature
            .clone()
            .sync_committee_bits
            .into_bytes()
            .into_vec()
            .as_slice()
            .try_into() {
                Ok(ba) => ba,
                Err(_) => { return Err(Box::new(beacon_rpc_client::ErrorOnUnwrapSignatureBit())); }
        };

        Ok(LightClientUpdate {
            attested_beacon_header: eth_types::eth2::BeaconBlockHeader {
                slot: attested_header.slot.as_u64(),
                proposer_index: attested_header.proposer_index,
                parent_root: eth_types::H256::from(attested_header.parent_root.0),
                state_root: eth_types::H256::from(attested_header.state_root.0),
                body_root: eth_types::H256::from(attested_header.body_root.0),
            },
            sync_aggregate: eth_types::eth2::SyncAggregate {
                sync_committee_bits: SyncCommitteeBits(sync_committee_bits),
                sync_committee_signature: serde_json::from_str::<SignatureBytes>(
                    &serde_json::to_string(&sync_committe_signature.sync_committee_signature)?,
                )?,
            },
            signature_slot,
            finality_update: FinalizedHeaderUpdate {
                header_update: HeaderUpdate {
                    beacon_header: eth_types::eth2::BeaconBlockHeader {
                        slot: finality_header.slot.as_u64(),
                        proposer_index: finality_header.proposer_index,
                        parent_root: eth_types::H256::from(finality_header.parent_root.0),
                        state_root: eth_types::H256::from(finality_header.state_root.0),
                        body_root: eth_types::H256::from(finality_header.body_root.0),
                    },
                    execution_block_hash: eth_types::H256::from(
                        finalized_block_eth1data_proof
                            .get_execution_block_hash()
                            .0
                            .to_vec(),
                    ),
                    execution_hash_branch: finalized_block_eth1data_proof
                        .get_proof()
                        .iter()
                        .copied()
                        .map(|x| eth_types::H256::from(x.0.to_vec()))
                        .collect(),
                },
                finality_branch: finality_branch
                    .into_iter()
                    .map(|x| eth_types::H256::from(x.0.to_vec()))
                    .collect(),
            },
            sync_committee_update: Option::<SyncCommitteeUpdate>::None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::beacon_rpc_client::BeaconRPCClient;
    use crate::hand_made_finality_light_client_update::HandMadeFinalityLightClientUpdate;

    const SIGNATURE_SLOT: u64 = 812638;
    const BEACON_ENDPOINT: &str = "https://lodestar-kiln.chainsafe.io";

    #[test]
    fn test_hand_made_finality_light_client_update() {
        let beacon_rpc_client = BeaconRPCClient::new(BEACON_ENDPOINT);
        let hand_made_light_client_update =
            HandMadeFinalityLightClientUpdate::get_finality_light_client_update(
                &beacon_rpc_client,
                SIGNATURE_SLOT,
            )
            .unwrap();
        let light_client_update = beacon_rpc_client.get_light_client_update(99).unwrap();

        assert_eq!(
            serde_json::to_string(&hand_made_light_client_update.finality_update).unwrap(),
            serde_json::to_string(&light_client_update.finality_update).unwrap()
        );
        assert_eq!(
            serde_json::to_string(&hand_made_light_client_update.signature_slot).unwrap(),
            serde_json::to_string(&light_client_update.signature_slot).unwrap()
        );
        assert_eq!(
            serde_json::to_string(&hand_made_light_client_update.attested_beacon_header).unwrap(),
            serde_json::to_string(&light_client_update.attested_beacon_header).unwrap()
        );
        assert_eq!(
            serde_json::to_string(&hand_made_light_client_update.sync_aggregate).unwrap(),
            serde_json::to_string(&light_client_update.sync_aggregate).unwrap()
        );
    }
}
