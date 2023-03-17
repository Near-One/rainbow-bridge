use bitvec::order::Lsb0;
use bitvec::prelude::BitVec;
use eth2_utility::consensus::{
    compute_domain, compute_signing_root, get_participant_pubkeys, Network, NetworkConfig,
    DOMAIN_SYNC_COMMITTEE, MIN_SYNC_COMMITTEE_PARTICIPANTS,
};
use eth_types::eth2::{BeaconBlockHeader, LightClientUpdate, SyncCommittee};
use eth_types::H256;
use std::error::Error;
use std::str::FromStr;
use types::{Hash256, Slot};

#[cfg(test)]
pub mod config_for_tests;

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
    ethereum_network: &str,
    light_client_update: &LightClientUpdate,
    sync_committee: SyncCommittee,
) -> Result<bool, Box<dyn Error>> {
    let ethereum_network = Network::from_str(ethereum_network)?;
    let config = NetworkConfig::new(&ethereum_network);

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
    use crate::config_for_tests::ConfigForTests;
    use crate::is_correct_finality_update;
    use eth_types::eth2::LightClientUpdate;
    use eth_types::eth2::SyncCommittee;

    fn get_config() -> ConfigForTests {
        ConfigForTests::load_from_toml("config_for_tests.toml".try_into().unwrap())
    }

    #[test]
    fn smoke_verify_finality_update() {
        let config = get_config();

        let light_client_updates: Vec<LightClientUpdate> = serde_json::from_str(
            &std::fs::read_to_string(config.path_to_light_client_updates)
                .expect("Unable to read file"),
        )
        .unwrap();

        let current_sync_committee: SyncCommittee = serde_json::from_str(
            &std::fs::read_to_string(config.path_to_current_sync_committee.clone())
                .expect("Unable to read file"),
        )
        .unwrap();
        let next_sync_committee: SyncCommittee = serde_json::from_str(
            &std::fs::read_to_string(config.path_to_next_sync_committee.clone())
                .expect("Unable to read file"),
        )
        .unwrap();

        assert!(is_correct_finality_update(
            &config.network_name,
            &light_client_updates[0],
            current_sync_committee
        )
        .unwrap());

        assert!(!is_correct_finality_update(
            &config.network_name,
            &light_client_updates[0],
            next_sync_committee
        )
        .unwrap());
    }
}
