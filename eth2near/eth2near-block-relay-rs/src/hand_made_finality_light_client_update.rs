use crate::beacon_block_body_merkle_tree::BeaconStateMerkleTree;
use crate::beacon_rpc_client::BeaconRPCClient;
use crate::execution_block_proof::ExecutionBlockProof;
use crate::relay_errors::{
    ErrorOnUnwrapSignatureBit, MissNextSyncCommittee, MissSyncAggregationError,
};
use eth_types::eth2::{
    FinalizedHeaderUpdate, HeaderUpdate, LightClientUpdate, SignatureBytes, SyncCommittee,
    SyncCommitteeBits, SyncCommitteeUpdate,
};
use eth_types::H256;
use std::error::Error;
use tree_hash::TreeHash;
use types::{BeaconBlockBody, BeaconBlockHeader, BeaconState, MainnetEthSpec};

pub struct HandMadeFinalityLightClientUpdate {}

impl HandMadeFinalityLightClientUpdate {
    pub fn get_finality_light_client_update(
        beacon_rpc_client: &BeaconRPCClient,
        attested_slot: u64,
        include_next_sync_committee: bool,
    ) -> Result<LightClientUpdate, Box<dyn Error>> {
        let signature_slot = beacon_rpc_client
            .get_non_empty_beacon_block_header(attested_slot + 1)?
            .slot
            .into();

        let signature_beacon_body =
            beacon_rpc_client.get_beacon_block_body_for_block_id(&format!("{}", signature_slot))?;
        let sync_committe_signature = signature_beacon_body
            .sync_aggregate()
            .map_err(|_| MissSyncAggregationError)?;

        let attested_header = beacon_rpc_client
            .get_beacon_block_header_for_block_id(&format!("{}", attested_slot))?;

        let beacon_state = beacon_rpc_client.get_beacon_state(&format!("{}", attested_slot))?;

        let finality_hash = beacon_state.finalized_checkpoint().root;
        let finality_header = beacon_rpc_client
            .get_beacon_block_header_for_block_id(&format!("{:?}", &finality_hash))?;

        let finalized_block_body = beacon_rpc_client
            .get_beacon_block_body_for_block_id(&format!("{:?}", &finality_hash))?;

        let sync_committee_bits: [u8; 64] = Self::get_sync_committee_bits(sync_committe_signature)?;

        Ok(LightClientUpdate {
            attested_beacon_header: Self::from_lighthouse_beacon_header(&attested_header),
            sync_aggregate: eth_types::eth2::SyncAggregate {
                sync_committee_bits: SyncCommitteeBits(sync_committee_bits),
                sync_committee_signature: serde_json::from_str::<SignatureBytes>(
                    &serde_json::to_string(&sync_committe_signature.sync_committee_signature)?,
                )?,
            },
            signature_slot,
            finality_update: Self::get_finality_update(
                &finality_header,
                &beacon_state,
                &finalized_block_body,
            )?,
            sync_committee_update: match include_next_sync_committee {
                false => Option::<SyncCommitteeUpdate>::None,
                true => Some(Self::get_next_sync_committee(
                    finality_header.slot.as_u64(),
                    beacon_rpc_client,
                )?),
            },
        })
    }
}

impl HandMadeFinalityLightClientUpdate {
    fn get_next_sync_committee(
        finality_slot: u64,
        beacon_rpc_client: &BeaconRPCClient,
    ) -> Result<SyncCommitteeUpdate, Box<dyn Error>> {
        let beacon_state = beacon_rpc_client.get_beacon_state(&format!("{}", finality_slot))?;
        let next_sync_committee = beacon_state
            .next_sync_committee()
            .map_err(|_| MissNextSyncCommittee)?;

        let beacon_state_merkle_tree = BeaconStateMerkleTree::new(&beacon_state);

        const BEACON_STATE_MERKLE_TREE_DEPTH: usize = 5;
        const BEACON_STATE_NEXT_SYNC_COMMITTEE_INDEX: usize = 23;

        let proof = beacon_state_merkle_tree.0.generate_proof(
            BEACON_STATE_NEXT_SYNC_COMMITTEE_INDEX,
            BEACON_STATE_MERKLE_TREE_DEPTH,
        );

        let next_sync_committee_branch = proof.1;

        let next_sync_committee_branch = next_sync_committee_branch
            .into_iter()
            .map(|x| eth_types::H256::from(x.0.to_vec()))
            .collect();

        let sync_committee = SyncCommittee {
            pubkeys: eth_types::eth2::SyncCommitteePublicKeys(
                next_sync_committee
                    .pubkeys
                    .iter()
                    .copied()
                    .map(|x| eth_types::eth2::PublicKeyBytes(x.serialize()))
                    .collect(),
            ),
            aggregate_pubkey: eth_types::eth2::PublicKeyBytes(
                next_sync_committee.aggregate_pubkey.serialize(),
            ),
        };

        Ok(SyncCommitteeUpdate {
            next_sync_committee: sync_committee,
            next_sync_committee_branch,
        })
    }

    fn from_lighthouse_beacon_header(
        beacon_header: &BeaconBlockHeader,
    ) -> eth_types::eth2::BeaconBlockHeader {
        eth_types::eth2::BeaconBlockHeader {
            slot: beacon_header.slot.as_u64(),
            proposer_index: beacon_header.proposer_index,
            parent_root: eth_types::H256::from(beacon_header.parent_root.0),
            state_root: eth_types::H256::from(beacon_header.state_root.0),
            body_root: eth_types::H256::from(beacon_header.body_root.0),
        }
    }

    fn get_sync_committee_bits(
        sync_committee_signature: &types::SyncAggregate<MainnetEthSpec>,
    ) -> Result<[u8; 64], Box<dyn Error>> {
        match sync_committee_signature
            .clone()
            .sync_committee_bits
            .into_bytes()
            .into_vec()
            .as_slice()
            .try_into()
        {
            Ok(ba) => Ok(ba),
            Err(_) => Err(Box::new(ErrorOnUnwrapSignatureBit)),
        }
    }

    fn get_finality_branch(
        beacon_state: &BeaconState<MainnetEthSpec>,
    ) -> Result<Vec<H256>, Box<dyn Error>> {
        const BEACON_STATE_MERKLE_TREE_DEPTH: usize = 5;
        const BEACON_STATE_FINALIZED_CHECKPOINT_INDEX: usize = 20;

        let beacon_state_merkle_tree = BeaconStateMerkleTree::new(&beacon_state);
        let mut proof = beacon_state_merkle_tree.0.generate_proof(
            BEACON_STATE_FINALIZED_CHECKPOINT_INDEX,
            BEACON_STATE_MERKLE_TREE_DEPTH,
        );

        let mut finality_branch = vec![beacon_state.finalized_checkpoint().epoch.tree_hash_root()];
        finality_branch.append(&mut proof.1);

        Ok(finality_branch
            .into_iter()
            .map(|x| eth_types::H256::from(x.0.to_vec()))
            .collect())
    }

    fn get_finality_update(
        finality_header: &BeaconBlockHeader,
        beacon_state: &BeaconState<MainnetEthSpec>,
        finalized_block_body: &BeaconBlockBody<MainnetEthSpec>,
    ) -> Result<FinalizedHeaderUpdate, Box<dyn Error>> {
        let finality_branch = Self::get_finality_branch(beacon_state)?;
        let finalized_block_eth1data_proof =
            ExecutionBlockProof::construct_from_beacon_block_body(finalized_block_body)?;

        Ok(FinalizedHeaderUpdate {
            header_update: HeaderUpdate {
                beacon_header: Self::from_lighthouse_beacon_header(finality_header),
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
            finality_branch,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::beacon_rpc_client::BeaconRPCClient;
    use crate::hand_made_finality_light_client_update::HandMadeFinalityLightClientUpdate;

    const ATTESTED_SLOT: u64 = 812637;
    const BEACON_ENDPOINT: &str = "https://lodestar-kiln.chainsafe.io";

    #[test]
    fn test_hand_made_finality_light_client_update() {
        let beacon_rpc_client = BeaconRPCClient::new(BEACON_ENDPOINT);
        let hand_made_light_client_update =
            HandMadeFinalityLightClientUpdate::get_finality_light_client_update(
                &beacon_rpc_client,
                ATTESTED_SLOT,
                true,
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
        assert_eq!(
            serde_json::to_string(&hand_made_light_client_update.sync_committee_update).unwrap(),
            serde_json::to_string(&light_client_update.sync_committee_update).unwrap()
        )
    }
}
