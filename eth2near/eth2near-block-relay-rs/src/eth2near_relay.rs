use crate::beacon_rpc_client::{BeaconRPCClient, ExecutionPayloadError, MissSyncCommitteeUpdate};
use std::cmp::max;
use std::error::Error;
use std::vec::Vec;
use contract_wrapper::contract_wrapper_trait::ContractWrapper;
use contract_wrapper::eth_client_contract::EthClientContract;
use eth_types::{BlockHeader, H256};
use eth_types::eth2::LightClientUpdate;
use crate::eth1_rpc_client::Eth1RPCClient;
use log::{info, warn, trace};
use crate::hand_made_finality_light_client_update::HandMadeFinalityLightClientUpdate;
use crate::config::Config;

pub struct Eth2NearRelay {
    beacon_rpc_client: BeaconRPCClient,
    eth1_rpc_client: Eth1RPCClient,
    eth_client_contract: EthClientContract,
    max_submitted_headers: u64,
    current_gap_between_finalized_and_signature_slot: u64,
    network: String,
}

impl Eth2NearRelay {
    pub fn init(config: &Config, contract_wrapper: Box<dyn ContractWrapper>) -> Self {
        info!(target: "relay", "=== Relay initialization === ");

        let eth2near_relay = Eth2NearRelay {
            beacon_rpc_client: BeaconRPCClient::new(&config.beacon_endpoint),
            eth1_rpc_client: Eth1RPCClient::new(&config.eth1_endpoint),
            eth_client_contract: EthClientContract::new(contract_wrapper),
            max_submitted_headers: config.total_submit_headers as u64,
            current_gap_between_finalized_and_signature_slot: 96,
            network: config.network.to_string(),
        };
        eth2near_relay.eth_client_contract.register().unwrap();
        eth2near_relay
    }

    pub fn run(&mut self) {
        info!(target: "relay", "=== Relay running ===");
        loop {
            info!(target: "relay", "== New relay loop ==");
            let last_eth2_slot_on_near: u64;
            let last_eth2_slot_on_eth_chain: u64;

            match self.get_last_slot() {
                Ok(slot) => last_eth2_slot_on_near = slot,
                Err(err) => {
                    warn!(target: "relay", "Fail to get last slot on NEAR. Error: {}", err);
                    continue;
                }
            }

            match self.beacon_rpc_client.get_last_slot_number() {
                Ok(slot) => last_eth2_slot_on_eth_chain = slot.as_u64(),
                Err(err) => {
                    warn!(target: "relay", "Fail to get last slot on Eth. Error: {}", err);
                    continue;
                }
            }

            info!(target: "relay", "Last slot on near = {}; last slot on eth = {}",
                  last_eth2_slot_on_near, last_eth2_slot_on_eth_chain);

            if last_eth2_slot_on_near < last_eth2_slot_on_eth_chain {
                info!(target: "relay", "= Creating headers batch =");
                let mut headers: Vec<BlockHeader> = vec![];
                let mut current_slot = last_eth2_slot_on_near + 1;
                while headers.len() < self.max_submitted_headers as usize && current_slot <= last_eth2_slot_on_eth_chain {
                    trace!(target: "relay", "Try add block header for slot={}, headers len={}/{}", current_slot, headers.len(), self.max_submitted_headers);
                    let mut count = 0;
                    loop {
                        if count > 0 {
                            trace!(target: "relay", "Error on extraction execution block header for slot = {}. Try again. Trying number {}", current_slot, count + 1);
                        }

                        if let Ok(block_number) = self.beacon_rpc_client.get_block_number_for_slot(types::Slot::new(current_slot)) {
                            if let Ok(eth1_header) = self.eth1_rpc_client.get_block_header_by_number(block_number) {
                                headers.push(eth1_header);
                                break;
                            }
                        }
                        count += 1;
                        if count > 2 {
                            trace!(target: "relay", "Block header for slot={} was not extracted. Skip!", current_slot);
                            break;
                        }
                    }
                    current_slot += 1;
                }

                for _ in 1..5 {
                    info!(target: "relay", "Try submit headers from slot={} to {} to NEAR", last_eth2_slot_on_near + 1, current_slot - 1);
                    match self.eth_client_contract.send_headers(&headers, current_slot - 1) {
                        Ok(()) => {
                            info!(target: "relay", "Successful headers submission!");
                            break;
                        }
                        Err(err) => warn!(target: "relay", "Error \"{}\" on headers submission!", err)
                    }
                }
                self.send_light_client_updates();
            }
        }
    }

    fn block_known_on_near(& self, slot: u64) -> Result<bool, Box<dyn Error>> {
        trace!(target: "relay", "Check if block with slot={} on NEAR", slot);
        match self.beacon_rpc_client.get_beacon_block_body_for_block_id(&format!("{}", slot)) {
            Ok(beacon_block_body) => {
                let hash: H256 = H256::from(beacon_block_body.execution_payload().map_err(|_| { ExecutionPayloadError() })?.execution_payload.block_hash.into_root().as_bytes());
                if self.eth_client_contract.is_known_block(&hash)? == true {
                    return Ok(true);
                } else {
                    trace!(target: "relay", "Block with slot={} not found on Near", slot);
                    return Ok(false);
                }
            }
            Err(err) => trace!(target: "relay", "Error \"{}\" in getting beacon block body for slot={}", err, slot)
        }
        return Ok(false);
    }

    fn search_slot_forward(&self, slot: u64) -> Result<u64, Box<dyn Error>> {
        let mut current_step = 1;
        while self.block_known_on_near(slot + current_step)? {
            current_step *= 2;
        }

        return self.search_slot_backward(slot, slot + current_step);
    }

    fn search_slot_backward(&self, start_slot: u64, last_slot: u64) -> Result<u64, Box<dyn Error>> {
        let mut start_slot = start_slot;
        let mut last_slot = last_slot;
        while start_slot + 1 < last_slot {
            let mid_slot = start_slot + (last_slot - start_slot)/2;
            if self.block_known_on_near(mid_slot)? {
                start_slot = mid_slot;
            } else {
                last_slot = mid_slot;
            }
        }
        return Ok(start_slot);
    }

    fn get_last_slot(& mut self) -> Result<u64, Box<dyn Error>> {
        trace!(target: "relay", "= Search for last slot on near =");

        let mut slot = self.eth_client_contract.get_last_submitted_slot();
        
        let finalized_slot = self.eth_client_contract.get_finalized_beacon_block_slot()?;
        trace!(target: "relay", "Finalized slot on near={}", finalized_slot);

        slot = max(finalized_slot, slot);
        trace!(target: "relay", "Init slot for search as {}", slot);

        if self.block_known_on_near(slot)? {
            return self.search_slot_forward(slot);
        } else {
            return self.search_slot_backward(finalized_slot, slot);
        }
    }

    fn verify_bls_signature_for_finality_update(&mut self, light_client_update: &LightClientUpdate) -> Result<bool, Box<dyn Error>> {
        let current_period = BeaconRPCClient::get_period_for_slot(light_client_update.attested_beacon_header.slot);
        let update_for_per_period = self.beacon_rpc_client.get_light_client_update(current_period - 1)?;
        let sync_committee = update_for_per_period.sync_committee_update.ok_or(MissSyncCommitteeUpdate())?.next_sync_committee;

        finality_update_verify::is_correct_finality_update(&self.network, light_client_update, sync_committee)
    }

    fn send_specific_light_cleint_update(&mut self, light_client_update: LightClientUpdate) {
        match self.eth_client_contract.is_known_block(&light_client_update.finality_update.header_update.execution_block_hash) {
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
                    match self.eth_client_contract.send_light_client_update(light_client_update) {
                        Ok(()) => {
                            info!(target: "relay", "Successful light client update submission!");
                            self.current_gap_between_finalized_and_signature_slot = 96;
                        },
                        Err(err) => warn!(target: "relay", "Fail to send light client update. Error: {}", err)
                    }
                } else {
                    trace!(target: "relay", "Finalized block for light client update is not found on NEAR. Skipping send light client update");
                }
            }
            Err(err) => trace!(target: "relay", "Fail on the is_known_block method. Skipping sending light client update. Error: {}", err)
        }
    }

    fn send_hand_made_light_client_update(&mut self, last_finalized_slot_on_near: u64) {
        let last_submitted_slot = self.eth_client_contract.get_last_submitted_slot();
        if (last_submitted_slot as i64) - (last_finalized_slot_on_near as i64) < (self.current_gap_between_finalized_and_signature_slot as i64) {
            info!(target: "relay", "Waiting for sending more headers to near. Skip sending light client update.");
            return;
        }

        let signature_slot = last_finalized_slot_on_near + self.current_gap_between_finalized_and_signature_slot;
        match HandMadeFinalityLightClientUpdate::get_finality_light_client_update(&self.beacon_rpc_client, signature_slot) {
            Ok(mut light_client_update) => {
                let finality_update_slot = light_client_update.finality_update.header_update.beacon_header.slot;
                if BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_near) != BeaconRPCClient::get_period_for_slot(finality_update_slot) {
                    let new_period = BeaconRPCClient::get_period_for_slot(finality_update_slot);
                    match self.beacon_rpc_client.get_light_client_update(new_period) {
                        Ok(light_client_update_for_period) => light_client_update.sync_committee_update = light_client_update_for_period.sync_committee_update,
                        Err(err) => {
                            trace!(target: "relay", "Error \"{}\" on getting light client update for period. Skipping sending light client update", err);
                            return;
                        }
                    }
                }
                self.send_specific_light_cleint_update(light_client_update);
            }
            Err(err) => {
                trace!(target: "relay", "Error \"{}\" on getting hand made light client update for attested slot={}.", err, signature_slot);
                self.current_gap_between_finalized_and_signature_slot += 1;
            }
        }
    }

    fn send_light_client_updates(&mut self) {
        info!(target: "relay", "= Sending light client update =");

        let finalized_block_hash: H256;
        match self.eth_client_contract.get_finalized_beacon_block_hash() {
            Ok(block_hash) => finalized_block_hash = block_hash,
            Err(err) => {
                warn!(target: "relay", "Error \"{}\" on getting finalized block hash. Skipping sending light client update", err);
                return;
            }
        }

        let last_finalized_slot_on_near: u64;
        match self.beacon_rpc_client.get_slot_by_beacon_block_root(finalized_block_hash) {
            Ok(last_finalized_slot) => last_finalized_slot_on_near = last_finalized_slot,
            Err(err) => {
                warn!(target: "relay", "Error \"{}\" on getting slot for finalized block hash. Skipping sending light client update", err);
                return;
            }
        }

        let last_submitted_slot = self.eth_client_contract.get_last_submitted_slot();

        if (last_submitted_slot as i64) - (last_finalized_slot_on_near as i64) < 32 {
            info!(target: "relay", "Light client update were send less then epoch ago. Skipping sending light client update");
            return;
        }

        let last_eth2_period_on_near_chain = BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_near);
        info!(target: "relay", "Last finalized slot/period on near={}/{}", last_finalized_slot_on_near, last_eth2_period_on_near_chain);

        let last_finalized_slot_on_eth: u64;
        match self.beacon_rpc_client.get_last_finalized_slot_number() {
            Ok(end_slot) => last_finalized_slot_on_eth = end_slot.as_u64(),
            Err(err) => {
                warn!(target: "relay", "Error \"{}\" on getting last finalized slot number on Ethereum. Skipping sending light client update", err);
                return;
            }
        }

        let end_period = BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_eth);
        info!(target: "relay", "Last finalized slot/period on ethereum={}/{}", last_finalized_slot_on_eth, end_period);

        if last_finalized_slot_on_eth - last_finalized_slot_on_near > 500 {
            info!(target: "relay", "Too big gap between slot of finalized block on Near and Eth. Sending hand made light client update");
            self.send_hand_made_light_client_update(last_finalized_slot_on_near);
            return;
        }

        if last_finalized_slot_on_eth <= last_finalized_slot_on_near {
            info!(target: "relay", "Last finalized slot on Eth equal to last finalized slot on NEAR. Skipping sending light client update.");
            return;
        }

        if end_period == last_eth2_period_on_near_chain {
            trace!(target: "relay", "Finalized period on Eth and Near are equal. Don't fetch sync commity update");
            match self.beacon_rpc_client.get_finality_light_client_update() {
                Ok(light_client_update) => self.send_specific_light_cleint_update(light_client_update),
                Err(err) => warn!(target: "relay", "Error \"{}\" on getting light client update. Skipping sending light client update", err)
            }
        } else {
            trace!(target: "relay", "Finalized period on Eth and Near are different. Fetching sync commity update");
            match self.beacon_rpc_client.get_finality_light_client_update_with_sync_commity_update() {
                Ok(light_client_update) => self.send_specific_light_cleint_update(light_client_update),
                Err(err) => warn!(target: "relay", "Error \"{}\" on getting light client update. Skipping sending light client update", err)
            }
        }
    }
}