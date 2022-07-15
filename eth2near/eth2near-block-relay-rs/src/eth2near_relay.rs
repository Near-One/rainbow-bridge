use crate::beacon_block_header_with_execution_data::BeaconBlockHeaderWithExecutionData;
use crate::beacon_rpc_client::BeaconRPCClient;
use crate::eth_client_contract::EthClientContract;
use std::cmp::min;
use std::vec::Vec;

pub struct Eth2NearRelay {
    beacon_rpc_client: BeaconRPCClient,
    eth_client_contract: EthClientContract,
    max_submitted_headers: u64,
}

impl Eth2NearRelay {
    pub fn init(eth_node_url: &str, start_slot: u64, out_dir: String, max_submitted_headers: u64) -> Self {
        Eth2NearRelay {
            beacon_rpc_client: BeaconRPCClient::new(eth_node_url),
            eth_client_contract: EthClientContract::new(start_slot, out_dir),
            max_submitted_headers,
        }
    }

    pub fn run(&mut self) {
        loop {
            let last_eth2_slot_on_near : u64 = self.eth_client_contract.get_last_slot();
            let last_eth2_slot_on_eth_chain : u64;

            if let Ok(slot) = self.beacon_rpc_client.get_last_finalized_slot_number() {
                last_eth2_slot_on_eth_chain = slot.as_u64();
            } else {
                continue
            }

            if last_eth2_slot_on_near < last_eth2_slot_on_eth_chain {
                let mut end_slot = min(last_eth2_slot_on_eth_chain,
                                       last_eth2_slot_on_near + self.max_submitted_headers);

                if last_eth2_slot_on_near < 5 {
                    end_slot = last_eth2_slot_on_near + 1;
                }

                let mut headers: Vec<BeaconBlockHeaderWithExecutionData> = vec![];
                for i in last_eth2_slot_on_near + 1 ..=end_slot {
                    println!("slot={}", i);
                    let mut count = 0;
                    loop {
                        if let Ok(beacon_block_header) = self.beacon_rpc_client.get_beacon_block_header_for_block_id(&format!("{}", i)) {
                            if let Ok(beacon_block_body) = self.beacon_rpc_client.get_beacon_block_body_for_block_id(&format!("{}", i)) {
                                if let Ok(beacon_block_header_with_execution_data) = BeaconBlockHeaderWithExecutionData::new(beacon_block_header, &beacon_block_body) {
                                    headers.push(beacon_block_header_with_execution_data);
                                }
                                break;
                            }
                        }
                        count += 1;
                        if count > 2 {
                            break;
                        }
                    }
                }
                self.eth_client_contract.send_headers(headers);
                self.send_light_client_updates(end_slot, last_eth2_slot_on_eth_chain);
            }
        }
    }

    fn send_light_client_updates(&mut self, end_slot: u64, last_eth2_slot_on_eth: u64) {
        let last_eth2_period_on_near_chain = self.eth_client_contract.get_last_period();
        let last_eth2_period_on_eth_chain = BeaconRPCClient::get_period_for_slot(last_eth2_slot_on_eth);

        let end_period = BeaconRPCClient::get_period_for_slot(end_slot);

        for period in last_eth2_period_on_near_chain + 1..=min(last_eth2_period_on_eth_chain - 1, end_period) {
            for _ in 0..5 {
                if let Ok(light_client_update) = self.beacon_rpc_client.get_light_client_update(period) {
                    self.eth_client_contract.send_light_client_update(light_client_update, period);
                    break;
                }
            }
        }

        if end_period == last_eth2_period_on_eth_chain {
            for _ in 0..5 {
                if let Ok(light_client_update) = self.beacon_rpc_client.get_light_client_update(end_period) {
                    self.eth_client_contract.send_light_client_update(light_client_update, end_period);
                    break;
                }
            }
        }
    }
}