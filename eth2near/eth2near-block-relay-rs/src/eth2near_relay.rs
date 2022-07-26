use crate::beacon_rpc_client::{BeaconRPCClient, ExecutionPayloadError};
use crate::eth_client_contract::EthClientContract;
use std::cmp::max;
use std::error::Error;
use std::vec::Vec;
use eth_types::{BlockHeader, H256};
use eth_types::eth2::LightClientUpdate;
use crate::eth1_rpc_client::Eth1RPCClient;
use log::{info, warn};
use crate::hand_made_finality_light_client_update::HandMadeFinalityLightClientUpdate;

pub struct Eth2NearRelay {
    beacon_rpc_client: BeaconRPCClient,
    eth1_rpc_client: Eth1RPCClient,
    eth_client_contract: EthClientContract,
    max_submitted_headers: u64,
    current_gap_between_finalized_and_attested_slot: u64,
}

impl Eth2NearRelay {
    pub fn init(eth_node_url: &str, eth1_endpoint: &str, start_slot: u64, max_submitted_headers: u32,
                near_endpoint: &str, signer_account_id: &str,
                path_to_signer_secret_key: &str, contract_account_id: &str) -> Self {
        info!(target: "relay", "=== Relay initialization === ");

        let eth2near_relay = Eth2NearRelay {
            beacon_rpc_client: BeaconRPCClient::new(eth_node_url),
            eth1_rpc_client: Eth1RPCClient::new(eth1_endpoint),
            eth_client_contract: EthClientContract::new(near_endpoint, signer_account_id,
                                                        path_to_signer_secret_key, contract_account_id,
                                                        start_slot),
            max_submitted_headers: max_submitted_headers as u64,
            current_gap_between_finalized_and_attested_slot: 96,
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
                    info!(target: "relay", "Try add block header for slot={}, headers len={}/{}", current_slot, headers.len(), self.max_submitted_headers);
                    let mut count = 0;
                    loop {
                        if count > 0 {
                            warn!(target: "relay", "Error on extraction execution block header for slot = {}. Try again. Trying number {}", current_slot, count + 1);
                        }

                        if let Ok(block_number) = self.beacon_rpc_client.get_block_number_for_slot(types::Slot::new(current_slot)) {
                            if let Ok(eth1_header) = self.eth1_rpc_client.get_block_header_by_number(block_number) {
                                headers.push(eth1_header);
                                break;
                            }
                        }
                        count += 1;
                        if count > 2 {
                            warn!(target: "relay", "Block header for slot={} was not extracted. Skip!", current_slot);
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

    fn get_last_slot(& mut self) -> Result<u64, Box<dyn Error>> {
        info!(target: "relay", "= Search for last slot on near =");

        let mut slot = self.eth_client_contract.get_last_submitted_slot();
        let finalized_block_hash = self.eth_client_contract.get_finalized_beacon_block_hash()?;
        let finalized_slot = self.beacon_rpc_client.get_slot_by_beacon_block_root(finalized_block_hash)?;
        info!(target: "relay", "Finalized slot on near={}", finalized_slot);

        slot = max(finalized_slot, slot);

        info!(target: "relay", "Init slot for search as {}", slot);

        while slot > finalized_slot {
            info!(target: "relay", "Check if block with slot={} on NEAR", slot);
            match self.beacon_rpc_client.get_beacon_block_body_for_block_id(&format!("{}", slot)) {
                Ok(beacon_block_body) => {
                    let hash: H256 = H256::from(beacon_block_body.execution_payload().map_err(|_| { ExecutionPayloadError() })?.execution_payload.block_hash.into_root().as_bytes());
                    if self.eth_client_contract.is_known_block(&hash)? == true {
                        break;
                    } else {
                        info!(target: "relay", "Block with slot={} not found on Near", slot)
                    }
                }
                Err(err) => warn!(target: "relay", "Error \"{}\" in getting beacon block body for slot={}", err, slot)
            }
            slot -= 1;
        }

        Ok(slot)
    }

    fn send_specific_light_cleint_update(&mut self, light_client_update: LightClientUpdate) {
        match self.eth_client_contract.is_known_block(&light_client_update.finality_update.header_update.execution_block_hash) {
            Ok(is_known_block) => {
                if is_known_block {
                    info!(target: "relay", "Sending light client update");
                    match self.eth_client_contract.send_light_client_update(light_client_update) {
                        Ok(()) => {
                            info!(target: "relay", "Successful light client update submission!");
                            self.current_gap_between_finalized_and_attested_slot = 96;
                        },
                        Err(err) => warn!(target: "relay", "Fail to send light client update. Error: {}", err)
                    }
                } else {
                    warn!(target: "relay", "Finalized block for light client update is not found on NEAR. Skipping send light client update");
                }
            }
            Err(err) => warn!(target: "relay", "Fail on the is_known_block method. Skipping sending light client update. Error: {}", err)
        }
    }

    fn send_hand_made_light_client_update(&mut self, last_finalized_slot_on_near: u64) {
        let last_submitted_slot = self.eth_client_contract.get_last_submitted_slot();
        if (last_submitted_slot as i64) - (last_finalized_slot_on_near as i64) < (self.current_gap_between_finalized_and_attested_slot as i64) {
            info!(target: "relay", "Waiting for sending more headers to near. Skip sending light client update.");
            return;
        }

        let attested_slot = last_finalized_slot_on_near + self.current_gap_between_finalized_and_attested_slot;
        match HandMadeFinalityLightClientUpdate::get_finality_light_client_update(&self.beacon_rpc_client, attested_slot) {
            Ok(light_client_update) => self.send_specific_light_cleint_update(light_client_update),
            Err(err) => {
                warn!(target: "relay", "Error \"{}\" on getting hand made light client update for attested slot={}.", err, attested_slot);
                self.current_gap_between_finalized_and_attested_slot += 1;
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

        if last_finalized_slot_on_eth - last_finalized_slot_on_near > 500 {
            info!(target: "relay", "Too big gap between slot of finalized block on Near and Eth. Sending hand made light client update");
            self.send_hand_made_light_client_update(last_finalized_slot_on_near);
            return;
        }

        let end_period = BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_eth);
        info!(target: "relay", "Last finalized slot/period on ethereum={}/{}", last_finalized_slot_on_eth, end_period);

        if last_finalized_slot_on_eth <= last_finalized_slot_on_near {
            info!(target: "relay", "Last finalized slot on Eth equal to last finalized slot on NEAR. Skipping sending light client update.");
            return;
        }

        if end_period == last_eth2_period_on_near_chain {
            info!(target: "relay", "Finalized period on Eth and Near are equal. Don't fetch sync commity update");
            match self.beacon_rpc_client.get_finality_light_client_update() {
                Ok(light_client_update) => self.send_specific_light_cleint_update(light_client_update),
                Err(err) => warn!(target: "relay", "Error \"{}\" on getting light client update. Skipping sending light client update", err)
            }
        } else {
            info!(target: "relay", "Finalized period on Eth and Near are different. Fetching sync commity update");
            match self.beacon_rpc_client.get_finality_light_client_update_with_sync_commity_update() {
                Ok(light_client_update) => self.send_specific_light_cleint_update(light_client_update),
                Err(err) => warn!(target: "relay", "Error \"{}\" on getting light client update. Skipping sending light client update", err)
            }
        }
    }
}