use crate::beacon_rpc_client::{BeaconRPCClient, ExecutionPayloadError};
use crate::eth_client_contract::EthClientContract;
use std::cmp::max;
use std::error::Error;
use std::vec::Vec;
use eth_types::{BlockHeader, H256};
use crate::eth1_rpc_client::Eth1RPCClient;
use log::{info, warn};

pub struct Eth2NearRelay {
    beacon_rpc_client: BeaconRPCClient,
    eth1_rpc_client: Eth1RPCClient,
    eth_client_contract: EthClientContract,
    max_submitted_headers: u64,
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
        };
        eth2near_relay.eth_client_contract.register().unwrap();
        eth2near_relay
    }

    pub fn run(&mut self) {
        info!(target: "relay", "=== Relay running ===");
        loop {
            info!(target: "relay", "== New relay loop ==");
            let last_eth2_slot_on_near : u64;
            let last_eth2_slot_on_eth_chain : u64;

            if let Ok(slot) = self.get_last_slot() {
                last_eth2_slot_on_near = slot;
            } else {
                warn!(target: "relay", "Fail to get last slot on NEAR");
                continue;
            }

            if let Ok(slot) = self.beacon_rpc_client.get_last_slot_number() {
                last_eth2_slot_on_eth_chain = slot.as_u64();
            } else {
                warn!(target: "relay", "Fail to get last slot on Eth");
                continue;
            }

            info!(target: "relay", "Last slot on near = {}; last slot on eth = {}",
                  last_eth2_slot_on_near, last_eth2_slot_on_eth_chain);

            if last_eth2_slot_on_near < last_eth2_slot_on_eth_chain {
                info!(target: "relay", "= Creating headers bunch =");
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
                    if let Ok(()) = self.eth_client_contract.send_headers(&headers, current_slot - 1) {
                        info!(target: "relay", "Successful headers submission!");
                        break;
                    } else {
                        warn!(target: "relay", "Error on headers submission!");
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
            if let Ok(beacon_block_body) = self.beacon_rpc_client.get_beacon_block_body_for_block_id(&format!("{}", slot)) {
                let hash: H256 = H256::from(beacon_block_body.execution_payload().map_err(|_| {ExecutionPayloadError()})?.execution_payload.block_hash.into_root().as_bytes());
                if self.eth_client_contract.is_known_block(&hash)? == true {
                    break;
                } else {
                    info!(target: "relay", "Block with slot={} not found on Near", slot)
                }
            } else {
                warn!(target: "relay", "Error in getting beacon block body for slot={}", slot);
            }
            slot -= 1;
        }

        Ok(slot)
    }

    fn send_light_client_updates(&mut self) {
        info!(target: "relay", "= Sending light client update =");

        if let Ok(finalized_block_hash) = self.eth_client_contract.get_finalized_beacon_block_hash() {
            if let Ok(last_finalized_slot_on_near) = self.beacon_rpc_client.get_slot_by_beacon_block_root(finalized_block_hash) {
                let last_eth2_period_on_near_chain = BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_near);
                info!(target: "relay", "Last finalized slot/period on near={}/{}", last_finalized_slot_on_near, last_eth2_period_on_near_chain);

                if let Ok(end_slot) = self.beacon_rpc_client.get_last_finalized_slot_number() {
                    let end_period = BeaconRPCClient::get_period_for_slot(end_slot.as_u64());
                    info!(target: "relay", "Last finalized slot/period on ethereum={}/{}", end_slot, end_period);

                    if end_slot <= last_finalized_slot_on_near {
                        info!(target: "relay", "Last finalized slot on Eth equal to last finalized slot on NEAR. Skipping sending light client update.");
                        return;
                    }

                    if end_period == last_eth2_period_on_near_chain {
                        info!(target: "relay", "Finalized period on Eth and Near are equal. Don't fetch sync commity update");
                        if let Ok(light_client_update) = self.beacon_rpc_client.get_finality_light_client_update() {
                            if let Ok(is_known_block) = self.eth_client_contract.is_known_block(&light_client_update.finality_update.header_update.execution_block_hash) {
                                if is_known_block {
                                    info!(target: "relay", "Sending light client update");
                                    if let Ok(()) = self.eth_client_contract.send_light_client_update(light_client_update) {
                                        info!(target: "relay", "Successful light client update submission!");
                                    } else {
                                        warn!(target: "relay", "Fail to send light client update");
                                    }
                                } else {
                                    warn!(target: "relay", "Finalized block for light client update is not found on NEAR. Skipping send light client update");
                                }
                            } else {
                                warn!(target: "relay", "Fail on the is_known_block method. Skipping sending light client update");
                            }
                        } else {
                            warn!(target: "relay", "Error on getting light client update. Skipping sending light client update");
                        }
                    } else {
                        info!(target: "relay", "Finalized period on Eth and Near are different. Fetching sync commity update");
                        if let Ok(light_client_update) = self.beacon_rpc_client.get_finality_light_client_update_with_sync_commity_update() {
                            if let Ok(is_known_block) = self.eth_client_contract.is_known_block(&light_client_update.finality_update.header_update.execution_block_hash) {
                                if is_known_block {
                                    info!(target: "relay", "Sending light client update");
                                    if let Ok(()) = self.eth_client_contract.send_light_client_update(light_client_update) {
                                        info!(target: "relay", "Successful light client update submission!");
                                    } else {
                                        warn!(target: "relay", "Fail to send light client update");
                                    }
                                } else {
                                    warn!(target: "relay", "Finalized block for light client update is not found on NEAR. Skipping send light client update");
                                }
                            } else {
                                warn!(target: "relay", "Fail on the is_known_block method. Skipping sending light client update");
                            }
                        } else {
                            warn!(target: "relay", "Error on getting light client update. Skipping sending light client update");
                        }
                    }
                } else {
                    warn!(target: "relay", "Error on getting last finalized slot number on Ethereum. Skipping sending light client update");
                }
            } else {
                warn!(target: "relay", "Error on getting slot for finalized block hash. Skipping sending light client update");
            }
        } else {
            warn!(target: "relay", "Error on getting finalized block hash. Skipping sending light client update");
        }
    }
}