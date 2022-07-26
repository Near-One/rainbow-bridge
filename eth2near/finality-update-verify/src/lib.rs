use eth_types::eth2::{LightClientUpdate, SyncCommittee};

pub fn is_correct_finality_update(network: &str, light_client_update: LightClientUpdate, sync_committee: SyncCommittee) -> bool {
    return true;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
