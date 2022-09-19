use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use bls;
use eth2_utility::consensus::{
    compute_domain, compute_signing_root, get_participant_pubkeys, Network, NetworkConfig,
    DOMAIN_SYNC_COMMITTEE, MIN_SYNC_COMMITTEE_PARTICIPANTS,
};
use eth_types::eth2::{BeaconBlockHeader, LightClientUpdate, SyncCommittee};
use eth_types::H256;
use std::error::Error;
use std::str::FromStr;
use types::{Hash256, Slot};

fn h256_to_hash256(hash: H256) -> Hash256 {
    Hash256::from_slice(hash.0.as_bytes())
}

fn tree_hash_h256_to_eth_type_h256(hash: tree_hash::Hash256) -> eth_types::H256 {
    eth_types::H256::from(hash.0.as_slice())
}

fn to_lighthouse_beacon_block_header(
    bridge_beacon_block_header: &BeaconBlockHeader,
) -> types::BeaconBlockHeader {
    types::BeaconBlockHeader {
        slot: Slot::from(bridge_beacon_block_header.slot),
        proposer_index: bridge_beacon_block_header.proposer_index,
        parent_root: h256_to_hash256(bridge_beacon_block_header.parent_root),
        state_root: h256_to_hash256(bridge_beacon_block_header.state_root),
        body_root: h256_to_hash256(bridge_beacon_block_header.body_root),
    }
}

pub fn is_correct_finality_update(
    network: &str,
    light_client_update: &LightClientUpdate,
    sync_committee: SyncCommittee,
) -> Result<bool, Box<dyn Error>> {
    let network = Network::from_str(network)?;
    let config = NetworkConfig::new(&network);

    let sync_committee_bits =
        BitVec::<u8, Lsb0>::from_slice(&light_client_update.sync_aggregate.sync_committee_bits.0);

    let sync_committee_bits_sum: u64 = sync_committee_bits.count_ones().try_into()?;
    if sync_committee_bits_sum < MIN_SYNC_COMMITTEE_PARTICIPANTS {
        return Ok(false);
    }
    if sync_committee_bits_sum * 3 < (sync_committee_bits.len() * 2).try_into()? {
        return Ok(false);
    }

    let participant_pubkeys =
        get_participant_pubkeys(&sync_committee.pubkeys.0, &sync_committee_bits);
    let fork_version = config
        .compute_fork_version_by_slot(light_client_update.signature_slot)
        .expect("Unsupported fork");
    let domain = compute_domain(
        DOMAIN_SYNC_COMMITTEE,
        fork_version,
        config.genesis_validators_root.into(),
    );

    let attested_beacon_header_root = tree_hash::TreeHash::tree_hash_root(
        &to_lighthouse_beacon_block_header(&light_client_update.attested_beacon_header),
    );
    let signing_root = compute_signing_root(
        tree_hash_h256_to_eth_type_h256(attested_beacon_header_root),
        domain,
    );

    let aggregate_signature = bls::AggregateSignature::deserialize(
        &light_client_update
            .sync_aggregate
            .sync_committee_signature
            .0,
    )
    .map_err(|_err| -> String { "Error on aggregate signature deserialization".to_string() })?;
    let mut pubkeys: Vec<bls::PublicKey> = vec![];
    for pubkey in participant_pubkeys {
        pubkeys.push(
            bls::PublicKey::deserialize(&pubkey.0)
                .map_err(|_err| -> String { "Error on public key deserialization".to_string() })?,
        );
    }

    Ok(aggregate_signature.fast_aggregate_verify(
        h256_to_hash256(signing_root),
        &pubkeys.iter().collect::<Vec<_>>(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::is_correct_finality_update;
    use eth2_to_near_relay::beacon_rpc_client::BeaconRPCClient;

    const BEACON_ENDPOINT: &str = "https://lodestar-kiln.chainsafe.io";
    const TIMEOUT_SECONDS: u64 = 30;
    const TIMEOUT_STATE_SECONDS: u64 = 1000;

    #[test]
    fn smoke_verify_finality_update_true() {
        let network = "kiln";
        let beacon_rpc_client =
            BeaconRPCClient::new(BEACON_ENDPOINT, TIMEOUT_SECONDS, TIMEOUT_STATE_SECONDS);
        let light_client_update_period_99 = beacon_rpc_client.get_light_client_update(99).unwrap();
        let light_client_update_period_100 =
            beacon_rpc_client.get_light_client_update(100).unwrap();

        assert!(is_correct_finality_update(
            network,
            &light_client_update_period_100,
            light_client_update_period_99
                .sync_committee_update
                .unwrap()
                .next_sync_committee
        )
        .unwrap());
    }

    #[test]
    fn smoke_verify_finality_update_false() {
        let network = "kiln";
        let beacon_rpc_client =
            BeaconRPCClient::new(BEACON_ENDPOINT, TIMEOUT_SECONDS, TIMEOUT_STATE_SECONDS);
        let light_client_update_period_99 = beacon_rpc_client.get_light_client_update(99).unwrap();
        let light_client_update_period_100 =
            beacon_rpc_client.get_light_client_update(100).unwrap();

        assert!(!is_correct_finality_update(
            network,
            &light_client_update_period_99,
            light_client_update_period_100
                .sync_committee_update
                .unwrap()
                .next_sync_committee
        )
        .unwrap());
    }
}
