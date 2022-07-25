use std::error::Error;
use eth_types::eth2::LightClientUpdate;
use tree_hash::TreeHash;
use crate::beacon_block_body_merkle_tree::BeaconStateMerkleTree;
use crate::beacon_rpc_client::BeaconRPCClient;

pub struct HandMadeFinalityLightClientUpdate {}

impl HandMadeFinalityLightClientUpdate {
    pub fn get_finality_light_client_update(beacon_rpc_client: &BeaconRPCClient,
                                            attested_slot: u64) -> Result<LightClientUpdate, Box<dyn Error>> {
        let attested_header = beacon_rpc_client.get_beacon_block_header_for_block_id(&format!("{}", attested_slot))?;
        let beacon_state = beacon_rpc_client.get_beacon_state(&format!("{}", attested_slot))?;
        let finality_hash = beacon_state.finalized_checkpoint().root;
        let finality_header = beacon_rpc_client.get_beacon_block_header_for_block_id(&format!("{:?}", &finality_hash));
        println!("attested_header: {:?}", attested_header);
        println!("finality_checkpoint: {:?}", beacon_state.finalized_checkpoint());
        println!("finality_hash: {:?}", finality_hash);
        println!("finality header: {:?}", finality_header);

        let beacon_state_merkle_tree = BeaconStateMerkleTree::new(&beacon_state);
        let mut proof = beacon_state_merkle_tree.0.generate_proof(20, 5);

        println!("Proof: {:?}", proof);

        let mut finality_branch = vec![beacon_state.finalized_checkpoint().epoch.tree_hash_root()];
        finality_branch.append(&mut proof.1);
        println!("Finality branch: {:?}", finality_branch);

        Err("not implemented")?
    }
}

#[cfg(test)]
mod tests {
    use crate::beacon_rpc_client::BeaconRPCClient;
    use crate::hand_made_finality_light_client_update::HandMadeFinalityLightClientUpdate;

    const ATTESTED_HEADER_SLOT: u64 = 812637;

    #[test]
    fn test_hand_made_finality_light_client_update() {
        let beacon_rpc_client = BeaconRPCClient::default();
        HandMadeFinalityLightClientUpdate::get_finality_light_client_update(&beacon_rpc_client, ATTESTED_HEADER_SLOT).unwrap();
    }
}
