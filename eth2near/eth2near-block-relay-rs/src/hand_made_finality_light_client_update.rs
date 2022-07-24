use std::error::Error;
use eth_types::eth2::LightClientUpdate;
use crate::beacon_rpc_client::BeaconRPCClient;

pub struct HandMadeFinalityLightClientUpdate {}

impl HandMadeFinalityLightClientUpdate {
    pub fn get_finality_light_client_update(beacon_rpc_client: &BeaconRPCClient,
                                            attested_slot: u64) -> Result<LightClientUpdate, Box<dyn Error>> {
        Err("not implemented")?
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hand_made_finality_light_client_update() {}
}
