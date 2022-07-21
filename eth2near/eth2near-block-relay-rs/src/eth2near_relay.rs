use crate::beacon_rpc_client::BeaconRPCClient;
use crate::eth_client_contract::EthClientContract;
use std::cmp::{max, min};
use std::vec::Vec;
use eth_types::{BlockHeader, H256};
use crate::eth1_rpc_client::Eth1RPCClient;

pub struct Eth2NearRelay {
    beacon_rpc_client: BeaconRPCClient,
    eth1_rpc_client: Eth1RPCClient,
    eth_client_contract: EthClientContract,
    max_submitted_headers: u64,
}

impl Eth2NearRelay {
    pub fn init(eth_node_url: &str, eth1_endpoint: &str, start_slot: u64, out_dir: String, max_submitted_headers: u32,
                near_endpoint: &str, signer_account_id: &str,
                path_to_signer_secret_key: &str, contract_account_id: &str) -> Self {
        let eth2near_relay = Eth2NearRelay {
            beacon_rpc_client: BeaconRPCClient::new(eth_node_url),
            eth1_rpc_client: Eth1RPCClient::new(eth1_endpoint),
            eth_client_contract: EthClientContract::new(near_endpoint, signer_account_id,
                                                        path_to_signer_secret_key, contract_account_id,
                                                        start_slot, out_dir),
            max_submitted_headers: max_submitted_headers as u64,
        };
        eth2near_relay.eth_client_contract.register();
        eth2near_relay
    }

    pub fn run(&mut self) {
        loop {
            let last_eth2_slot_on_near : u64 = self.get_last_slot();
            let last_eth2_slot_on_eth_chain : u64;

            if let Ok(slot) = self.beacon_rpc_client.get_last_finalized_slot_number() {
                last_eth2_slot_on_eth_chain = slot.as_u64();
            } else {
                continue
            }

            if last_eth2_slot_on_near < last_eth2_slot_on_eth_chain {
                let mut headers: Vec<BlockHeader> = vec![];
                let mut current_slot = last_eth2_slot_on_near + 1;
                while headers.len() < self.max_submitted_headers as usize && current_slot <= last_eth2_slot_on_eth_chain {
                    println!("slot={}, headers len={}", current_slot, headers.len());
                    let mut count = 0;
                    loop {
                        if let Ok(block_number) = self.beacon_rpc_client.get_block_number_for_slot(types::Slot::new(current_slot)) {
                            if let Ok(eth1_header) = self.eth1_rpc_client.get_block_header_by_number(block_number) {
                                headers.push(eth1_header);
                                break;
                            }
                        }
                        count += 1;
                        if count > 2 {
                            break;
                        }
                    }
                    current_slot += 1;
                }

                for _ in 1..5 {
                    if let Ok(()) = self.eth_client_contract.send_headers(&headers, last_eth2_slot_on_eth_chain + 1, current_slot - 1) {
                        break;
                    }
                }
                self.send_light_client_updates();
            }
        }
    }

    fn get_last_slot(& mut self) -> u64 {
        let mut slot = self.eth_client_contract.get_last_submitted_slot();
        let finalized_block_hash = self.eth_client_contract.get_finalized_beacon_block_hash();
        let finalized_slot = self.beacon_rpc_client.get_slot_by_beacon_block_root(finalized_block_hash).unwrap();

        slot = max(finalized_slot, slot);

        while slot > finalized_slot {
            println!("search last slot; current slot={}", slot);
            if let Ok(beacon_block_body) = self.beacon_rpc_client.get_beacon_block_body_for_block_id(&format!("{}", slot)) {
                let hash: H256 = H256::from(beacon_block_body.execution_payload().unwrap().execution_payload.block_hash.into_root().as_bytes());
                if self.eth_client_contract.is_known_block(&hash) == true {
                    break;
                }
            }
            slot -= 1;
        }

        return slot;
    }

    fn send_light_client_updates(&mut self) {
        let finalized_block_hash = self.eth_client_contract.get_finalized_beacon_block_hash();
        if let Ok(last_finalized_slot_on_near) = self.beacon_rpc_client.get_slot_by_beacon_block_root(finalized_block_hash) {
            let last_eth2_period_on_near_chain = BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_near);
            if let Ok(end_slot) = self.beacon_rpc_client.get_last_finalized_slot_number() {
                let end_period = BeaconRPCClient::get_period_for_slot(end_slot.as_u64());

                if end_slot <= last_finalized_slot_on_near {
                    return;
                }

                if end_period == last_eth2_period_on_near_chain {
                    if let Ok(light_client_update) = self.beacon_rpc_client.get_finality_light_client_update() {
                        if self.eth_client_contract.is_known_block(&light_client_update.finality_update.header_update.execution_block_hash) {
                            self.eth_client_contract.send_light_client_update(light_client_update, end_period);
                        }
                    }
                } else {
                    if let Ok(light_client_update) = self.beacon_rpc_client.get_finality_light_client_update_with_sync_commity_update() {
                        if self.eth_client_contract.is_known_block(&light_client_update.finality_update.header_update.execution_block_hash) {
                            self.eth_client_contract.send_light_client_update(light_client_update, end_period);
                        }
                    }
                }
            }
        }
    }
}