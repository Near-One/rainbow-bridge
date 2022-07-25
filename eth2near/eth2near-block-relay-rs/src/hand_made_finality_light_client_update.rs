use std::error::Error;
use eth_types::eth2::LightClientUpdate;
use crate::beacon_rpc_client::BeaconRPCClient;

pub struct HandMadeFinalityLightClientUpdate {}

impl HandMadeFinalityLightClientUpdate {
    pub fn get_finality_light_client_update(beacon_rpc_client: &BeaconRPCClient,
                                            attested_slot: u64) -> Result<LightClientUpdate, Box<dyn Error>> {
        let attested_header = beacon_rpc_client.get_beacon_block_header_for_block_id(&format!("{}", attested_slot))?;
        let finality_hash = beacon_rpc_client.get_finality_checkpoint_root(attested_slot)?;
        let finality_header = beacon_rpc_client.get_beacon_block_header_for_block_id(&serde_json::to_string(&finality_hash)?);
        println!("attested_header: {:?}", attested_header);
        println!("finality_hash: {:?}", finality_hash);
        println!("finality_header: {:?}", finality_header);
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
        HandMadeFinalityLightClientUpdate::get_finality_light_client_update(&beacon_rpc_client, ATTESTED_HEADER_SLOT);
    }
}
