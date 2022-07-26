use eth_types::eth2::{LightClientUpdate, SyncCommittee};

pub fn is_correct_finality_update(network: &str, light_client_update: LightClientUpdate, sync_committee: SyncCommittee) -> bool {
    return true;
}

#[cfg(test)]
mod tests {
    use eth2_to_near_relay::beacon_rpc_client::BeaconRPCClient;
    use crate::is_correct_finality_update;

    #[test]
    fn smoke_verify_finality_update_true() {
        let network = "kiln";
        let beacon_rpc_client = BeaconRPCClient::default();
        let light_client_update_period_99 = beacon_rpc_client.get_light_client_update(99).unwrap();
        let light_client_update_period_100 = beacon_rpc_client.get_light_client_update(100).unwrap();

        assert!(is_correct_finality_update(network, light_client_update_period_100, light_client_update_period_99.sync_committee_update.unwrap().next_sync_committee));
    }

    #[test]
    fn smoke_verify_finality_update_false() {
        let network = "kiln";
        let beacon_rpc_client = BeaconRPCClient::default();
        let light_client_update_period_99 = beacon_rpc_client.get_light_client_update(99).unwrap();
        let light_client_update_period_100 = beacon_rpc_client.get_light_client_update(100).unwrap();

        assert!(!is_correct_finality_update(network, light_client_update_period_99, light_client_update_period_100.sync_committee_update.unwrap().next_sync_committee));
    }
}
