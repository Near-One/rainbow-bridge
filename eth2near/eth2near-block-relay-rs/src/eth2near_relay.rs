use crate::beacon_rpc_client::BeaconRPCClient;
use crate::config::Config;
use crate::eth1_rpc_client::Eth1RPCClient;
use crate::hand_made_finality_light_client_update::HandMadeFinalityLightClientUpdate;
use crate::relay_errors::{MissSyncCommitteeUpdate};
use crate::last_slot_searcher::LastSlotSearcher;
use contract_wrapper::eth_client_contract_trait::EthClientContractTrait;
use eth_types::eth2::LightClientUpdate;
use eth_types::{BlockHeader, H256};
use log::{debug, info, trace, warn};
use std::error::Error;
use std::vec::Vec;

const ONE_EPOCH_IN_SLOTS: u64 = 32;

macro_rules! skip_fail {
    ($res:expr, $msg:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                warn!(target: "relay", "{}. Error: {}", $msg, e);
                continue;
            }
        }
    };
}

macro_rules! return_on_fail {
    ($res:expr, $msg:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                warn!(target: "relay", "{}. Error: {}", $msg, e);
                return;
            }
        }
    };
}

pub struct Eth2NearRelay {
    beacon_rpc_client: BeaconRPCClient,
    eth1_rpc_client: Eth1RPCClient,
    eth_client_contract: Box<dyn EthClientContractTrait>,
    max_submitted_headers: u64,
    current_gap_between_finalized_and_attested_slot: u64,
    network: String,
    light_client_updates_submission_frequency_in_epochs: i64,
    max_blocks_for_finalization: u64,
    near_network_name: String,
    last_slot_searcher: LastSlotSearcher,
}

impl Eth2NearRelay {
    pub fn init(
        config: &Config,
        eth_contract: Box<dyn EthClientContractTrait>,
        enable_binsearch: bool,
        register_relay: bool,
    ) -> Self {
        info!(target: "relay", "=== Relay initialization === ");

        let eth2near_relay = Eth2NearRelay {
            beacon_rpc_client: BeaconRPCClient::new(&config.beacon_endpoint),
            eth1_rpc_client: Eth1RPCClient::new(&config.eth1_endpoint),
            eth_client_contract: eth_contract,
            max_submitted_headers: config.total_submit_headers as u64,
            current_gap_between_finalized_and_attested_slot:
                Self::get_gap_between_finalized_and_attested_slot(
                    config.light_client_updates_submission_frequency_in_epochs as u64),
            network: config.network.to_string(),
            light_client_updates_submission_frequency_in_epochs: config
                .light_client_updates_submission_frequency_in_epochs,
            max_blocks_for_finalization: config.max_blocks_for_finalization,
            near_network_name: config.near_network_id.to_string(),
            last_slot_searcher: LastSlotSearcher::new(enable_binsearch),
        };

        if register_relay {
            eth2near_relay
                .eth_client_contract
                .register_submitter()
                .unwrap();
        }

        eth2near_relay
    }

    pub fn run(&mut self) {
        info!(target: "relay", "=== Relay running ===");
        loop {
            info!(target: "relay", "== New relay loop ==");

            let last_eth2_slot_on_eth_chain: u64 = skip_fail!(self.beacon_rpc_client.get_last_slot_number(),  "Fail to get last slot on Eth").as_u64();
            let mut last_eth2_slot_on_near: u64 = skip_fail!(self.last_slot_searcher.get_last_slot(last_eth2_slot_on_eth_chain, &self.beacon_rpc_client, &self.eth_client_contract), "Fail to get last slot on NEAR");

            info!(target: "relay", "Last slot on near = {}; last slot on eth = {}",
                  last_eth2_slot_on_near, last_eth2_slot_on_eth_chain);

            if last_eth2_slot_on_near < last_eth2_slot_on_eth_chain {
                info!(target: "relay", "= Creating headers batch =");
                let (headers, current_slot) = self.get_n_execution_blocks(last_eth2_slot_on_near + 1, last_eth2_slot_on_eth_chain);
                self.submit_execution_blocks(headers, current_slot, &mut last_eth2_slot_on_near);
                self.send_light_client_updates(last_eth2_slot_on_near);
            }
        }
    }

    fn get_n_execution_blocks(&self, start_slot: u64, last_eth2_slot_on_eth_chain: u64) -> (Vec<BlockHeader>, u64) {
        let mut headers: Vec<BlockHeader> = vec![];
        let mut current_slot = start_slot;

        while headers.len() < self.max_submitted_headers as usize
            && current_slot <= last_eth2_slot_on_eth_chain
        {
            debug!(target: "relay", "Try add block header for slot={}, headers len={}/{}", current_slot, headers.len(), self.max_submitted_headers);
            loop {
                if let Ok(eth1_header) = self.get_execution_block_by_slot(current_slot) {
                    headers.push(eth1_header);
                    break;
                }
            }
            current_slot += 1;
        }

        (headers, current_slot)
    }

    fn submit_execution_blocks(&mut self, headers: Vec<BlockHeader>, current_slot: u64, last_eth2_slot_on_near: &mut u64) {
        for _ in 1..5 {
            info!(target: "relay", "Try submit headers from slot={} to {} to NEAR", *last_eth2_slot_on_near + 1, current_slot - 1);
            let execution_outcome = skip_fail!(self
                .eth_client_contract
                .send_headers(&headers, current_slot - 1), "Error on header submission");

            *last_eth2_slot_on_near = current_slot - 1;
            info!(target: "relay", "Successful headers submission! Transaction URL: https://explorer.{}.near.org/transactions/{}",
                                  self.near_network_name, execution_outcome.transaction.hash);
            break;
        }
    }

    // get the slot numbers between the last submitted slot and attested slot for next update
    // if we sending updates once in 'update_submission_frequency' epochs
    // `update_submission_frequency * ONE_EPOCH_IN_SLOTS` -- gap in slots between two finalized
    //  blocks in neighboring updates.
    // `2 * ONE_EPOCH_IN_SLOTS` -- gap between finalized and attested block.
    // `1` -- expected gap between attested block slot and signature slot
    fn get_gap_between_finalized_and_attested_slot(update_submission_frequency: u64) -> u64 {
        const EXPECTED_EPOCHS_BETWEEN_HEAD_AND_FINALIZED_BLOCKS: u64 = 2;
        const EXPECTED_SLOTS_BETWEEN_ATTESTED_AND_SIGNATURE_SLOTS: u64 = 1;

        update_submission_frequency * ONE_EPOCH_IN_SLOTS
            + EXPECTED_EPOCHS_BETWEEN_HEAD_AND_FINALIZED_BLOCKS * ONE_EPOCH_IN_SLOTS
            + EXPECTED_SLOTS_BETWEEN_ATTESTED_AND_SIGNATURE_SLOTS
    }

    fn verify_bls_signature_for_finality_update(
        &mut self,
        light_client_update: &LightClientUpdate,
    ) -> Result<bool, Box<dyn Error>> {
        let current_period =
            BeaconRPCClient::get_period_for_slot(light_client_update.attested_beacon_header.slot);
        let update_for_per_period = self
            .beacon_rpc_client
            .get_light_client_update(current_period - 1)?;
        let sync_committee = update_for_per_period
            .sync_committee_update
            .ok_or(MissSyncCommitteeUpdate)?
            .next_sync_committee;

        finality_update_verify::is_correct_finality_update(
            &self.network,
            light_client_update,
            sync_committee,
        )
    }

    fn get_execution_block_by_slot(&self, slot: u64) -> Result<BlockHeader, Box<dyn Error>> {
        match self
            .beacon_rpc_client
            .get_block_number_for_slot(types::Slot::new(slot)) {
            Ok(block_number) => {
                return self
                    .eth1_rpc_client
                    .get_block_header_by_number(block_number);
            },
            Err(err) => Err(err),
        }
    }
}

// Implementation of functions for submitting light client updates
impl Eth2NearRelay {
    fn send_light_client_updates(&mut self, last_submitted_slot: u64) {
        info!(target: "relay", "= Sending light client update =");

        let finalized_block_hash: H256 = return_on_fail!(self
            .eth_client_contract
            .get_finalized_beacon_block_hash(), "Error on getting finalized block hash. Skipping sending light client update");

        let last_finalized_slot_on_near: u64 = return_on_fail!(self
            .beacon_rpc_client
            .get_slot_by_beacon_block_root(finalized_block_hash),
            "Error on getting slot for finalized block hash. Skipping sending light client update");

        if (last_submitted_slot as i64) - (last_finalized_slot_on_near as i64)
            < (ONE_EPOCH_IN_SLOTS as i64) * self.light_client_updates_submission_frequency_in_epochs
        {
            info!(target: "relay", "Light client update were send less then {} epochs ago. Skipping sending light client update", self.light_client_updates_submission_frequency_in_epochs);
            return;
        }

        let last_eth2_period_on_near_chain =
            BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_near);
        info!(target: "relay", "Last finalized slot/period on near={}/{}", last_finalized_slot_on_near, last_eth2_period_on_near_chain);

        let last_finalized_slot_on_eth: u64 = return_on_fail!(self
            .beacon_rpc_client
            .get_last_finalized_slot_number(),
            "Error on getting last finalized slot number on Ethereum. Skipping sending light client update").as_u64();

        let end_period = BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_eth);
        info!(target: "relay", "Last finalized slot/period on ethereum={}/{}", last_finalized_slot_on_eth, end_period);

        if last_finalized_slot_on_eth - last_finalized_slot_on_near
            >= self.max_blocks_for_finalization
        {
            info!(target: "relay", "Too big gap between slot of finalized block on Near and Eth. Sending hand made light client update");
            self.send_hand_made_light_client_update(
                last_finalized_slot_on_near,
                last_submitted_slot,
            );
            return;
        }

        if last_finalized_slot_on_eth <= last_finalized_slot_on_near {
            info!(target: "relay", "Last finalized slot on Eth equal to last finalized slot on NEAR. Skipping sending light client update.");
            return;
        }

        if end_period == last_eth2_period_on_near_chain {
            debug!(target: "relay", "Finalized period on Eth and Near are equal. Don't fetch sync commity update");
            let light_client_update= return_on_fail!(self.beacon_rpc_client.get_finality_light_client_update(),
                "Error on getting light client update. Skipping sending light client update");
            self.send_specific_light_cleint_update(light_client_update);
        } else {
            debug!(target: "relay", "Finalized period on Eth and Near are different. Fetching sync commity update");
            let light_client_update = return_on_fail!(self
                .beacon_rpc_client
                .get_finality_light_client_update_with_sync_commity_update(),
                "Error on getting light client update. Skipping sending light client update"
            );
            self.send_specific_light_cleint_update(light_client_update);
        }
    }

    fn send_hand_made_light_client_update(
        &mut self,
        last_finalized_slot_on_near: u64,
        last_submitted_slot: u64,
    ) {
        trace!(target: "relay", "last_finalized_slot_on_near {}", last_finalized_slot_on_near);

        if (last_submitted_slot as i64) - (last_finalized_slot_on_near as i64)
            < (ONE_EPOCH_IN_SLOTS as i64 * self.light_client_updates_submission_frequency_in_epochs)
        {
            info!(target: "relay", "Waiting for sending more headers to near. Skip sending light client update.");
            return;
        }

        let attested_slot =
            last_finalized_slot_on_near + self.current_gap_between_finalized_and_attested_slot;

        let attested_slot: u64 = return_on_fail!(self
            .beacon_rpc_client
            .get_non_empty_beacon_block_header(attested_slot), "Error on getting attested slot").slot.into();

        trace!(target: "relay", "Chosen attested slot {}", attested_slot);

        match HandMadeFinalityLightClientUpdate::get_finality_light_client_update(
            &self.beacon_rpc_client,
            attested_slot,
            BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_near)
                != BeaconRPCClient::get_period_for_slot(attested_slot),
        ) {
            Ok(light_client_update) => {
                let finality_update_slot = light_client_update
                    .finality_update
                    .header_update
                    .beacon_header
                    .slot;

                if finality_update_slot <= last_finalized_slot_on_near {
                    info!(target: "relay", "Finality update slot for hand made light client update <= last finality update on near. Increment gap for attested slot and skipping light client update.");
                    self.current_gap_between_finalized_and_attested_slot += ONE_EPOCH_IN_SLOTS;
                    return;
                }

                trace!(target: "relay", "Hand made light client update: {:?}", light_client_update);

                self.send_specific_light_cleint_update(light_client_update);
            },
            Err(err) => {
                debug!(target: "relay", "Error \"{}\" on getting hand made light client update for attested slot={}.", err, attested_slot);
                self.current_gap_between_finalized_and_attested_slot += 1;
            }
        }
    }

    fn send_specific_light_cleint_update(&mut self, light_client_update: LightClientUpdate) {
        let is_known_block = return_on_fail!(self.eth_client_contract.is_known_block(
            &light_client_update
                .finality_update
                .header_update
                .execution_block_hash,
        ), "Fail on the is_known_block method. Skipping sending light client update");

        if is_known_block {
            let verification_result = return_on_fail!(self.verify_bls_signature_for_finality_update(&light_client_update),  "Error on bls verification. Skip sending the light client update");
            if verification_result {
                info!(target: "relay", "PASS bls signature verification!");
            } else {
                warn!(target: "relay", "NOT PASS bls signature verification. Skip sending this light client update");
                return;
            }

            info!(target: "relay", "Sending light client update");

            let execution_outcome = return_on_fail!(self
                        .eth_client_contract
                        .send_light_client_update(light_client_update), "Fail to send light client update");
            info!(target: "relay", "Successful light client update submission! Transaction URL: https://explorer.{}.near.org/transactions/{}",
                                  self.near_network_name, execution_outcome.transaction.hash);
            self.current_gap_between_finalized_and_attested_slot =
                Self::get_gap_between_finalized_and_attested_slot(
                    self.light_client_updates_submission_frequency_in_epochs as u64,
                );
        } else {
            debug!(target: "relay", "Finalized block for light client update is not found on NEAR. Skipping send light client update");
        }
    }
}

#[cfg(test)]
mod tests {
    use eth_types::BlockHeader;
    use eth_types::eth2::LightClientUpdate;
    use log::LevelFilter;
    use crate::eth2near_relay::Eth2NearRelay;
    use crate::beacon_rpc_client::BeaconRPCClient;
    use crate::hand_made_finality_light_client_update::HandMadeFinalityLightClientUpdate;
    use crate::logger::SimpleLogger;
    use crate::relay_errors::NoBlockForSlotError;
    use crate::test_utils::get_relay;

    #[test]
    #[ignore]
    fn test_send_specific_light_client_update() {
        let mut relay = get_relay(true, true);
        let mut slot = relay.eth_client_contract.get_finalized_beacon_block_slot().unwrap();
        slot += 1;

        let mut blocks: Vec<BlockHeader> = vec![];
        while slot <= 1099392 {
            if let Ok(block) = relay.get_execution_block_by_slot(slot) {
                blocks.push(block)
            }
            slot += 1;
        }

        relay.eth_client_contract.send_headers(&blocks, 1099392).unwrap();
        let finalized_slot = relay.eth_client_contract.get_finalized_beacon_block_slot().unwrap();
        assert_eq!(finalized_slot, 1099360);

        const PATH_TO_LIGHT_CLIENT_UPDATES: &str = "../contract_wrapper/data/light_client_updates_kiln_1099394-1099937.json";
        let light_client_updates: Vec<LightClientUpdate> = serde_json::from_str(
            &std::fs::read_to_string(PATH_TO_LIGHT_CLIENT_UPDATES).expect("Unable to read file"),
        ).unwrap();
        relay.send_specific_light_cleint_update(light_client_updates[1].clone());

        let finalized_slot = relay.eth_client_contract.get_finalized_beacon_block_slot().unwrap();
        assert_eq!(finalized_slot, 1099392);
    }


    #[test]
    #[ignore]
    fn test_hand_made_light_client_update() {
        log::set_boxed_logger(Box::new(SimpleLogger))
            .map(|()| log::set_max_level(LevelFilter::Trace))
            .unwrap();

        let mut relay = get_relay(true, true);
        let mut slot = relay.eth_client_contract.get_finalized_beacon_block_slot().unwrap();
        slot += 1;

        let mut blocks: Vec<BlockHeader> = vec![];
        while slot <= 1099392 {
            if let Ok(block) = relay.get_execution_block_by_slot(slot) {
                blocks.push(block)
            }
            slot += 1;
        }

        relay.eth_client_contract.send_headers(&blocks, 1099392).unwrap();
        let finalized_slot = relay.eth_client_contract.get_finalized_beacon_block_slot().unwrap();
        assert_eq!(finalized_slot, 1099360);

        relay.send_hand_made_light_client_update(finalized_slot, 1099392);

        let finalized_slot = relay.eth_client_contract.get_finalized_beacon_block_slot().unwrap();
        assert_eq!(finalized_slot, 1099392);
    }

    #[test]
    #[ignore]
    fn test_send_light_client_update() {
        let mut relay = get_relay(true, false);
        let finality_slot = relay.eth_client_contract.get_finalized_beacon_block_slot().unwrap();
        let mut slot = finality_slot + 1;

        let mut finality_slot_on_eth = relay.beacon_rpc_client.get_last_finalized_slot_number().unwrap().as_u64();

        let mut blocks: Vec<BlockHeader> = vec![];
        while finality_slot == finality_slot_on_eth || slot <= finality_slot_on_eth {
            if let Ok(block) = relay.get_execution_block_by_slot(slot) {
                blocks.push(block)
            }
            slot += 1;

            finality_slot_on_eth = relay.beacon_rpc_client.get_last_finalized_slot_number().unwrap().as_u64();
        }

        relay.eth_client_contract.send_headers(&blocks, finality_slot_on_eth).unwrap();

        relay.send_light_client_updates(finality_slot_on_eth);

        let new_finalized_slot = relay.eth_client_contract.get_finalized_beacon_block_slot().unwrap();
        assert_ne!(finality_slot, new_finalized_slot);
    }

    #[test]
    fn test_get_execution_block_by_slot() {
        let mut relay = get_relay(true, true);
        relay.get_execution_block_by_slot(1099363).unwrap();
        if let Err(err) = relay.get_execution_block_by_slot(1099364) {
            if let None = err.downcast_ref::<NoBlockForSlotError>() {
                panic!("Wrong error type for slot without block");
            }
        } else {
            panic!("Return execution block for slot without block");
        }

        relay.beacon_rpc_client = BeaconRPCClient::new("http://httpstat.us/504/");
        if let Err(err) = relay.get_execution_block_by_slot(1099364) {
            if let Some(_) = err.downcast_ref::<NoBlockForSlotError>() {
                panic!("Wrong error type for unworking network");
            }
        } else {
            panic!("Return execution block in unworking network");
        }
    }

    #[test]
    fn test_verify_bls_signature() {
        let mut relay = get_relay(true, true);

        const PATH_TO_LIGHT_CLIENT_UPDATES: &str = "../contract_wrapper/data/light_client_updates_kiln_1099394-1099937.json";
        let mut light_client_updates: Vec<LightClientUpdate> = serde_json::from_str(
            &std::fs::read_to_string(PATH_TO_LIGHT_CLIENT_UPDATES).expect("Unable to read file"),
        ).unwrap();

        assert!(relay.verify_bls_signature_for_finality_update(&light_client_updates[1]).unwrap());
        light_client_updates[1].attested_beacon_header = light_client_updates[0].attested_beacon_header.clone();

        assert!(!relay.verify_bls_signature_for_finality_update(&light_client_updates[1]).unwrap());
    }

    #[test]
    fn test_get_gap_between_finalized_and_signature_slot() {
        let beacon_rpc_client = BeaconRPCClient::new("https://lodestar-kiln.chainsafe.io");
        let gap = Eth2NearRelay::get_gap_between_finalized_and_attested_slot(1);
        let finalized_slot = 1099488;
        let attested_slot = finalized_slot + gap;

        match HandMadeFinalityLightClientUpdate::get_finality_light_client_update(&beacon_rpc_client, attested_slot, false) {
            Ok(light_client_update) => {
                let finality_update_slot = light_client_update
                    .finality_update
                    .header_update
                    .beacon_header
                    .slot;

                assert!(finality_update_slot > finalized_slot);
            },
            Err(_) => { panic!("Error on get light client update"); }
        }
    }
}
