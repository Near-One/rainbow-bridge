use crate::beacon_rpc_client::BeaconRPCClient;
use crate::config::Config;
use crate::eth1_rpc_client::Eth1RPCClient;
use crate::hand_made_finality_light_client_update::HandMadeFinalityLightClientUpdate;
use crate::relay_errors::{ExecutionPayloadError, MissSyncCommitteeUpdate};
use contract_wrapper::contract_wrapper_trait::ContractWrapper;
use contract_wrapper::eth_client_contract::EthClientContract;
use eth_types::eth2::LightClientUpdate;
use eth_types::{BlockHeader, H256};
use log::{debug, info, trace, warn};
use std::cmp::{max, min};
use std::error::Error;
use std::vec::Vec;

const ONE_EPOCH_IN_SLOTS: u64 = 32;

pub struct Eth2NearRelay {
    beacon_rpc_client: BeaconRPCClient,
    eth1_rpc_client: Eth1RPCClient,
    eth_client_contract: EthClientContract,
    max_submitted_headers: u64,
    current_gap_between_finalized_and_signature_slot: u64,
    network: String,
    light_client_updates_submission_frequency_in_epochs: i64,
    max_blocks_for_finalization: u64,
    enable_binsearch: bool,
    near_network_name: String,
}

impl Eth2NearRelay {
    pub fn init(
        config: &Config,
        contract_wrapper: Box<dyn ContractWrapper>,
        enable_binsearch: bool,
        register_relay: bool,
    ) -> Self {
        info!(target: "relay", "=== Relay initialization === ");

        let eth2near_relay = Eth2NearRelay {
            beacon_rpc_client: BeaconRPCClient::new(&config.beacon_endpoint),
            eth1_rpc_client: Eth1RPCClient::new(&config.eth1_endpoint),
            eth_client_contract: EthClientContract::new(contract_wrapper),
            max_submitted_headers: config.total_submit_headers as u64,
            current_gap_between_finalized_and_signature_slot:
                Self::get_gap_between_finalized_and_signature_slot(
                    config.light_client_updates_submission_frequency_in_epochs as u64,
                ),
            network: config.network.to_string(),
            light_client_updates_submission_frequency_in_epochs: config
                .light_client_updates_submission_frequency_in_epochs,
            max_blocks_for_finalization: config.max_blocks_for_finalization,
            enable_binsearch,
            near_network_name: config.near_network_id.to_string(),
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

            let last_eth2_slot_on_eth_chain: u64 =
                match self.beacon_rpc_client.get_last_slot_number() {
                    Ok(slot) => slot.as_u64(),
                    Err(err) => {
                        warn!(target: "relay", "Fail to get last slot on Eth. Error: {}", err);
                        continue;
                    }
                };

            let last_eth2_slot_on_near: u64 = match self.get_last_slot(last_eth2_slot_on_eth_chain)
            {
                Ok(slot) => slot,
                Err(err) => {
                    warn!(target: "relay", "Fail to get last slot on NEAR. Error: {}", err);
                    continue;
                }
            };

            info!(target: "relay", "Last slot on near = {}; last slot on eth = {}",
                  last_eth2_slot_on_near, last_eth2_slot_on_eth_chain);

            if last_eth2_slot_on_near < last_eth2_slot_on_eth_chain {
                info!(target: "relay", "= Creating headers batch =");
                let mut headers: Vec<BlockHeader> = vec![];
                let mut current_slot = last_eth2_slot_on_near + 1;
                while headers.len() < self.max_submitted_headers as usize
                    && current_slot <= last_eth2_slot_on_eth_chain
                {
                    debug!(target: "relay", "Try add block header for slot={}, headers len={}/{}", current_slot, headers.len(), self.max_submitted_headers);
                    let mut count = 0;
                    loop {
                        if count > 0 {
                            debug!(target: "relay", "Error retrieving execution block header for slot = {}. Try again. Trying number {}", current_slot, count + 1);
                        }

                        if let Ok(block_number) = self
                            .beacon_rpc_client
                            .get_block_number_for_slot(types::Slot::new(current_slot))
                        {
                            if let Ok(eth1_header) = self
                                .eth1_rpc_client
                                .get_block_header_by_number(block_number)
                            {
                                headers.push(eth1_header);
                                break;
                            }
                        }
                        count += 1;
                        if count > 2 {
                            debug!(target: "relay", "Block header for slot={} was not extracted. Skip!", current_slot);
                            break;
                        }
                    }
                    current_slot += 1;
                }

                for _ in 1..5 {
                    info!(target: "relay", "Try submit headers from slot={} to {} to NEAR", last_eth2_slot_on_near + 1, current_slot - 1);
                    match self
                        .eth_client_contract
                        .send_headers(&headers, current_slot - 1)
                    {
                        Ok(transaction_id) => {
                            info!(target: "relay", "Successful headers submission! Transaction URL: https://explorer.{}.near.org/transactions/{}", self.near_network_name, transaction_id);
                            break;
                        }
                        Err(err) => {
                            warn!(target: "relay", "Error \"{}\" on headers submission!", err)
                        }
                    }
                }
                self.send_light_client_updates();
            }
        }
    }

    // get the slot numbers between the last submitted slot and signature slot for next update
    // if we sending updates once in 'update_submission_frequency' epochs
    // `update_submission_frequency * ONE_EPOCH_IN_SLOTS` -- gap in slots between two finalized
    //  blocks in neighboring updates.
    // `2 * ONE_EPOCH_IN_SLOTS` -- gap between finalized and attested block.
    // `1` -- expected gap between attested block slot and signature slot
    fn get_gap_between_finalized_and_signature_slot(update_submission_frequency: u64) -> u64 {
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
}

// Implementation of functions for submitting light client updates
impl Eth2NearRelay {
    fn send_light_client_updates(&mut self) {
        info!(target: "relay", "= Sending light client update =");

        let finalized_block_hash: H256 = match self
            .eth_client_contract
            .get_finalized_beacon_block_hash()
        {
            Ok(block_hash) => block_hash,
            Err(err) => {
                warn!(target: "relay", "Error \"{}\" on getting finalized block hash. Skipping sending light client update", err);
                return;
            }
        };

        let last_finalized_slot_on_near: u64 = match self
            .beacon_rpc_client
            .get_slot_by_beacon_block_root(finalized_block_hash)
        {
            Ok(last_finalized_slot) => last_finalized_slot,
            Err(err) => {
                warn!(target: "relay", "Error \"{}\" on getting slot for finalized block hash. Skipping sending light client update", err);
                return;
            }
        };

        let last_submitted_slot = self.eth_client_contract.get_last_submitted_slot();

        if (last_submitted_slot as i64) - (last_finalized_slot_on_near as i64)
            < 32 * self.light_client_updates_submission_frequency_in_epochs
        {
            info!(target: "relay", "Light client update were send less then {} epochs ago. Skipping sending light client update", self.light_client_updates_submission_frequency_in_epochs);
            return;
        }

        let last_eth2_period_on_near_chain =
            BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_near);
        info!(target: "relay", "Last finalized slot/period on near={}/{}", last_finalized_slot_on_near, last_eth2_period_on_near_chain);

        let last_finalized_slot_on_eth: u64 = match self
            .beacon_rpc_client
            .get_last_finalized_slot_number()
        {
            Ok(end_slot) => end_slot.as_u64(),
            Err(err) => {
                warn!(target: "relay", "Error \"{}\" on getting last finalized slot number on Ethereum. Skipping sending light client update", err);
                return;
            }
        };

        let end_period = BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_eth);
        info!(target: "relay", "Last finalized slot/period on ethereum={}/{}", last_finalized_slot_on_eth, end_period);

        if last_finalized_slot_on_eth - last_finalized_slot_on_near
            >= self.max_blocks_for_finalization
        {
            info!(target: "relay", "Too big gap between slot of finalized block on Near and Eth. Sending hand made light client update");
            self.send_hand_made_light_client_update(last_finalized_slot_on_near);
            return;
        }

        if last_finalized_slot_on_eth <= last_finalized_slot_on_near {
            info!(target: "relay", "Last finalized slot on Eth equal to last finalized slot on NEAR. Skipping sending light client update.");
            return;
        }

        if end_period == last_eth2_period_on_near_chain {
            debug!(target: "relay", "Finalized period on Eth and Near are equal. Don't fetch sync commity update");
            match self.beacon_rpc_client.get_finality_light_client_update() {
                Ok(light_client_update) => {
                    self.send_specific_light_cleint_update(light_client_update)
                }
                Err(err) => {
                    warn!(target: "relay", "Error \"{}\" on getting light client update. Skipping sending light client update", err)
                }
            }
        } else {
            debug!(target: "relay", "Finalized period on Eth and Near are different. Fetching sync commity update");
            match self
                .beacon_rpc_client
                .get_finality_light_client_update_with_sync_commity_update()
            {
                Ok(light_client_update) => {
                    self.send_specific_light_cleint_update(light_client_update)
                }
                Err(err) => {
                    warn!(target: "relay", "Error \"{}\" on getting light client update. Skipping sending light client update", err)
                }
            }
        }
    }

    fn send_hand_made_light_client_update(&mut self, last_finalized_slot_on_near: u64) {
        let last_submitted_slot = self.eth_client_contract.get_last_submitted_slot();
        if (last_submitted_slot as i64) - (last_finalized_slot_on_near as i64)
            < (self.current_gap_between_finalized_and_signature_slot as i64)
        {
            info!(target: "relay", "Waiting for sending more headers to near. Skip sending light client update.");
            return;
        }

        let signature_slot =
            last_finalized_slot_on_near + self.current_gap_between_finalized_and_signature_slot;
        trace!(target: "relay", "Chosen signature slot {}", signature_slot);

        match HandMadeFinalityLightClientUpdate::get_finality_light_client_update(
            &self.beacon_rpc_client,
            signature_slot,
        ) {
            Ok(mut light_client_update) => {
                let finality_update_slot = light_client_update
                    .finality_update
                    .header_update
                    .beacon_header
                    .slot;

                if finality_update_slot <= last_finalized_slot_on_near {
                    info!(target: "relay", "Finality update slot for hand made light client update <= last finality update on near. Increment gap for signature slot and skipping light client update.");
                    self.current_gap_between_finalized_and_signature_slot += ONE_EPOCH_IN_SLOTS;
                    return;
                }

                if BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_near)
                    != BeaconRPCClient::get_period_for_slot(finality_update_slot)
                {
                    let new_period = BeaconRPCClient::get_period_for_slot(finality_update_slot);
                    match self.beacon_rpc_client.get_light_client_update(new_period) {
                        Ok(light_client_update_for_period) => {
                            light_client_update.sync_committee_update =
                                light_client_update_for_period.sync_committee_update
                        }
                        Err(err) => {
                            debug!(target: "relay", "Error \"{}\" on getting light client update for period. Skipping sending light client update", err);
                            return;
                        }
                    }
                }

                trace!(target: "relay", "Hand made light client update: {:?}", light_client_update);

                self.send_specific_light_cleint_update(light_client_update);
            }
            Err(err) => {
                debug!(target: "relay", "Error \"{}\" on getting hand made light client update for attested slot={}.", err, signature_slot);
                self.current_gap_between_finalized_and_signature_slot += 1;
            }
        }
    }

    fn send_specific_light_cleint_update(&mut self, light_client_update: LightClientUpdate) {
        match self.eth_client_contract.is_known_block(
            &light_client_update
                .finality_update
                .header_update
                .execution_block_hash,
        ) {
            Ok(is_known_block) => {
                if is_known_block {
                    match self.verify_bls_signature_for_finality_update(&light_client_update) {
                        Ok(verification_result) => {
                            if verification_result {
                                info!(target: "relay", "PASS bls signature verification!");
                            } else {
                                warn!(target: "relay", "NOT PASS bls signature verification. Skip sending this light client update");
                                return;
                            }
                        }
                        Err(err) => {
                            warn!(target: "relay", "Error \"{}\" on bls verification. Skip sending the light client update.", err);
                            return;
                        }
                    }

                    info!(target: "relay", "Sending light client update");
                    match self
                        .eth_client_contract
                        .send_light_client_update(light_client_update)
                    {
                        Ok(transaction_id) => {
                            info!(target: "relay", "Successful light client update submission! Transaction URL: https://explorer.{}.near.org/transactions/{}", self.near_network_name, transaction_id);
                            self.current_gap_between_finalized_and_signature_slot =
                                Self::get_gap_between_finalized_and_signature_slot(
                                    self.light_client_updates_submission_frequency_in_epochs as u64,
                                );
                        }
                        Err(err) => {
                            warn!(target: "relay", "Fail to send light client update. Error: {}", err)
                        }
                    }
                } else {
                    debug!(target: "relay", "Finalized block for light client update is not found on NEAR. Skipping send light client update");
                }
            }
            Err(err) => {
                debug!(target: "relay", "Fail on the is_known_block method. Skipping sending light client update. Error: {}", err)
            }
        }
    }
}

// Implementation of functions for searching last slot on NEAR contract
impl Eth2NearRelay {
    fn get_last_slot(&mut self, last_eth_slot: u64) -> Result<u64, Box<dyn Error>> {
        debug!(target: "relay", "= Search for last slot on near =");

        let finalized_slot = self.eth_client_contract.get_finalized_beacon_block_slot()?;
        trace!(target: "relay", "Finalized slot on near={}", finalized_slot);

        let last_submitted_slot = self.eth_client_contract.get_last_submitted_slot();
        trace!(target: "relay", "Last submitted slot={}", last_submitted_slot);

        let slot = max(finalized_slot, last_submitted_slot);
        trace!(target: "relay", "Init slot for search as {}", slot);

        return if self.enable_binsearch {
            self.binary_slot_search(slot, finalized_slot, last_eth_slot)
        } else {
            self.linear_slot_search(slot, finalized_slot, last_eth_slot)
        };
    }

    fn linear_slot_search(
        &self,
        slot: u64,
        finalized_slot: u64,
        last_eth_slot: u64,
    ) -> Result<u64, Box<dyn Error>> {
        return if slot == finalized_slot || self.block_known_on_near(slot)? {
            Ok(self.linear_search_forward(slot, last_eth_slot))
        } else {
            Ok(self.linear_search_backward(finalized_slot, slot))
        };
    }

    fn binary_slot_search(
        &self,
        slot: u64,
        finalized_slot: u64,
        last_eth_slot: u64,
    ) -> Result<u64, Box<dyn Error>> {
        return if slot == finalized_slot || self.block_known_on_near(slot)? {
            self.binsearch_slot_forward(slot, last_eth_slot + 1)
        } else {
            self.binsearch_slot_range(finalized_slot, slot)
        };
    }

    fn binsearch_slot_forward(&self, slot: u64, max_slot: u64) -> Result<u64, Box<dyn Error>> {
        let mut current_step = 1;
        let mut prev_slot = slot;
        while slot + current_step < max_slot {
            match self.block_known_on_near(slot + current_step) {
                Ok(true) => {
                    prev_slot = slot + current_step;
                    current_step = min(current_step * 2, max_slot - slot);
                }
                Ok(false) => break,
                Err(_) => {
                    let (slot_id, slot_on_near) =
                        self.find_left_non_error_slot(slot + current_step + 1, max_slot);
                    if slot_on_near {
                        prev_slot = slot_id;
                        current_step = min(current_step * 2, max_slot - slot);
                    } else {
                        break;
                    }
                }
            }
        }

        self.binsearch_slot_range(prev_slot, slot + current_step)
    }

    fn binsearch_slot_range(&self, start_slot: u64, last_slot: u64) -> Result<u64, Box<dyn Error>> {
        let mut start_slot = start_slot;
        let mut last_slot = last_slot;
        while start_slot + 1 < last_slot {
            let mid_slot = start_slot + (last_slot - start_slot) / 2;
            match self.block_known_on_near(mid_slot) {
                Ok(true) => start_slot = mid_slot,
                Ok(false) => last_slot = mid_slot,
                Err(_) => {
                    let (left_slot, is_left_slot_on_near) =
                        self.find_left_non_error_slot(mid_slot + 1, last_slot);
                    if is_left_slot_on_near {
                        start_slot = left_slot;
                    } else {
                        last_slot = mid_slot;
                    }
                }
            }
        }

        Ok(start_slot)
    }

    fn linear_search_forward(&self, slot: u64, max_slot: u64) -> u64 {
        let mut slot = slot;
        while slot < max_slot {
            match self.block_known_on_near(slot + 1) {
                Ok(true) => slot += 1,
                Ok(false) => break,
                Err(_) => slot += 1,
            }
        }

        slot
    }

    fn linear_search_backward(&self, start_slot: u64, last_slot: u64) -> u64 {
        let mut slot = last_slot;

        while slot > start_slot {
            match self.block_known_on_near(slot) {
                Ok(true) => break,
                Ok(false) => slot -= 1,
                Err(_) => slot -= 1,
            }
        }

        slot
    }

    fn find_left_non_error_slot(&self, left_slot: u64, right_slot: u64) -> (u64, bool) {
        let mut slot = left_slot;
        while slot < right_slot {
            match self.block_known_on_near(slot) {
                Ok(v) => return (slot, v),
                Err(_) => slot += 1,
            };
        }

        (slot, false)
    }

    fn block_known_on_near(&self, slot: u64) -> Result<bool, Box<dyn Error>> {
        trace!(target: "relay", "Check if block with slot={} on NEAR", slot);
        match self
            .beacon_rpc_client
            .get_beacon_block_body_for_block_id(&format!("{}", slot))
        {
            Ok(beacon_block_body) => {
                let hash: H256 = H256::from(
                    beacon_block_body
                        .execution_payload()
                        .map_err(|_| ExecutionPayloadError)?
                        .execution_payload
                        .block_hash
                        .into_root()
                        .as_bytes(),
                );

                if self.eth_client_contract.is_known_block(&hash)? {
                    trace!(target: "relay", "Block with slot={} was found on NEAR", slot);
                    Ok(true)
                } else {
                    trace!(target: "relay", "Block with slot={} not found on Near", slot);
                    Ok(false)
                }
            }
            Err(err) => {
                trace!(target: "relay", "Error \"{}\" in getting beacon block body for slot={}", err, slot);
                Err(err)?
            }
        }
    }
}
