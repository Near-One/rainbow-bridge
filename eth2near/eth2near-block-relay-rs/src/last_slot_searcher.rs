use crate::beacon_rpc_client::BeaconRPCClient;
use crate::relay_errors::{ExecutionPayloadError, NoBlockForSlotError};
use contract_wrapper::eth_client_contract_trait::EthClientContractTrait;
use eth_types::H256;
use log::{debug, trace, info};
use std::cmp::{max, min};
use std::error::Error;

pub struct LastSlotSearcher {
    enable_binsearch: bool,
}

// Implementation of functions for searching last slot on NEAR contract
impl LastSlotSearcher {
    pub fn new(enable_binsearch: bool) -> LastSlotSearcher {
        LastSlotSearcher { enable_binsearch }
    }

    pub fn get_last_slot(
        &mut self,
        last_eth_slot: u64,
        beacon_rpc_client: &BeaconRPCClient,
        eth_client_contract: &Box<dyn EthClientContractTrait>,
    ) -> Result<u64, Box<dyn Error>> {
        info!(target: "relay", "= Search for last slot on near =");

        let finalized_slot = eth_client_contract.get_finalized_beacon_block_slot()?;
        let finalized_number = beacon_rpc_client.get_block_number_for_slot(types::Slot::new(finalized_slot))?;
        info!(target: "relay", "Finalized slot/block_number on near={}/{}", finalized_slot, finalized_number);

        let last_submitted_slot = eth_client_contract.get_last_submitted_slot();
        trace!(target: "relay", "Last submitted slot={}", last_submitted_slot);

        let slot = max(finalized_slot, last_submitted_slot);
        trace!(target: "relay", "Init slot for search as {}", slot);

        return if self.enable_binsearch {
            self.binary_slot_search(
                slot,
                finalized_slot,
                last_eth_slot,
                beacon_rpc_client,
                eth_client_contract,
            )
        } else {
            self.linear_slot_search(
                slot,
                finalized_slot,
                last_eth_slot,
                beacon_rpc_client,
                eth_client_contract,
            )
        };
    }

    // Search for the slot before the first unknown slot on NEAR
    // Assumptions:
    //     (1) start_slot is known on NEAR
    //     (2) last_slot is unknown on NEAR
    // Return error in case of problem with network connection
    fn binary_slot_search(
        &self,
        slot: u64,
        finalized_slot: u64,
        last_eth_slot: u64,
        beacon_rpc_client: &BeaconRPCClient,
        eth_client_contract: &Box<dyn EthClientContractTrait>,
    ) -> Result<u64, Box<dyn Error>> {
        if slot == finalized_slot {
            return self.binsearch_slot_forward(
                slot,
                last_eth_slot + 1,
                beacon_rpc_client,
                eth_client_contract,
            );
        }

        match self.block_known_on_near(slot, beacon_rpc_client, eth_client_contract) {
            Ok(true) => self.binsearch_slot_forward(
                slot,
                last_eth_slot + 1,
                beacon_rpc_client,
                eth_client_contract,
            ),
            Ok(false) => self.binsearch_slot_range(
                finalized_slot,
                slot,
                beacon_rpc_client,
                eth_client_contract,
            ),
            Err(err) => match err.downcast_ref::<NoBlockForSlotError>() {
                Some(_) => {
                    let (left_slot, slot_on_near) = self.find_left_non_error_slot(
                        slot + 1,
                        last_eth_slot + 1,
                        1,
                        beacon_rpc_client,
                        eth_client_contract,
                    );
                    match slot_on_near {
                        true => self.binsearch_slot_forward(
                            left_slot,
                            last_eth_slot + 1,
                            beacon_rpc_client,
                            eth_client_contract,
                        ),
                        false => self.binsearch_slot_range(
                            finalized_slot,
                            slot,
                            beacon_rpc_client,
                            eth_client_contract,
                        ),
                    }
                }
                None => Err(err),
            },
        }
    }

    // Search for the slot before the first unknown slot on NEAR
    // Assumptions:
    // (1) start_slot is known on NEAR
    // (2) last_slot is unknown on NEAR
    // Return error in case of problem with network connection
    fn binsearch_slot_forward(
        &self,
        slot: u64,
        max_slot: u64,
        beacon_rpc_client: &BeaconRPCClient,
        eth_client_contract: &Box<dyn EthClientContractTrait>,
    ) -> Result<u64, Box<dyn Error>> {
        let mut current_step = 1;
        let mut prev_slot = slot;
        while slot + current_step < max_slot {
            match self.block_known_on_near(
                slot + current_step,
                beacon_rpc_client,
                eth_client_contract,
            ) {
                Ok(true) => {
                    prev_slot = slot + current_step;
                    current_step = min(current_step * 2, max_slot - slot);
                }
                Ok(false) => break,
                Err(err) => match err.downcast_ref::<NoBlockForSlotError>() {
                    Some(_) => {
                        let (slot_id, slot_on_near) = self.find_left_non_error_slot(
                            slot + current_step - 1,
                            prev_slot,
                            -1,
                            beacon_rpc_client,
                            eth_client_contract,
                        );
                        if slot_on_near {
                            prev_slot = slot_id;
                            current_step = min(current_step * 2, max_slot - slot);
                        } else {
                            current_step = slot_id - slot;
                            break;
                        }
                    }
                    None => return Err(err),
                },
            }
        }

        self.binsearch_slot_range(
            prev_slot,
            slot + current_step,
            beacon_rpc_client,
            eth_client_contract,
        )
    }

    // Search for the slot before the first unknown slot on NEAR
    // Assumptions:
    // (1) start_slot is known on NEAR
    // (2) last_slot is unknown on NEAR
    // Return error in case of problem with network connection
    fn binsearch_slot_range(
        &self,
        start_slot: u64,
        last_slot: u64,
        beacon_rpc_client: &BeaconRPCClient,
        eth_client_contract: &Box<dyn EthClientContractTrait>,
    ) -> Result<u64, Box<dyn Error>> {
        let mut start_slot = start_slot;
        let mut last_slot = last_slot;
        while start_slot + 1 < last_slot {
            let mid_slot = start_slot + (last_slot - start_slot) / 2;
            match self.block_known_on_near(mid_slot, beacon_rpc_client, eth_client_contract) {
                Ok(true) => start_slot = mid_slot,
                Ok(false) => last_slot = mid_slot,
                Err(err) => match err.downcast_ref::<NoBlockForSlotError>() {
                    Some(_) => {
                        let (left_slot, is_left_slot_on_near) = self.find_left_non_error_slot(
                            mid_slot - 1,
                            start_slot,
                            -1,
                            beacon_rpc_client,
                            eth_client_contract,
                        );
                        if is_left_slot_on_near {
                            start_slot = mid_slot;
                        } else {
                            last_slot = left_slot;
                        }
                    }
                    None => return Err(err),
                },
            }
        }

        Ok(start_slot)
    }

    // Returns the last slot known with block known on NEAR
    // Slot -- expected last known slot
    // finalized_slot -- last finalized slot on NEAR, assume as known slot
    // last_eth_slot -- head slot on Eth
    fn linear_slot_search(
        &self,
        slot: u64,
        finalized_slot: u64,
        last_eth_slot: u64,
        beacon_rpc_client: &BeaconRPCClient,
        eth_client_contract: &Box<dyn EthClientContractTrait>,
    ) -> Result<u64, Box<dyn Error>> {
        if slot == finalized_slot {
            return Ok(self.linear_search_forward(
                slot,
                last_eth_slot,
                beacon_rpc_client,
                eth_client_contract,
            ));
        }

        match self.block_known_on_near(slot, beacon_rpc_client, eth_client_contract) {
            Ok(true) => Ok(self.linear_search_forward(
                slot,
                last_eth_slot,
                beacon_rpc_client,
                eth_client_contract,
            )),
            Ok(false) => Ok(self.linear_search_backward(
                finalized_slot,
                slot,
                beacon_rpc_client,
                eth_client_contract,
            )),
            Err(err) => match err.downcast_ref::<NoBlockForSlotError>() {
                Some(_) => {
                    let left_slot = self.linear_search_forward(
                        slot,
                        last_eth_slot,
                        beacon_rpc_client,
                        eth_client_contract,
                    );
                    if left_slot > slot {
                        Ok(left_slot)
                    } else {
                        Ok(self.linear_search_backward(
                            finalized_slot,
                            slot,
                            beacon_rpc_client,
                            eth_client_contract,
                        ))
                    }
                }
                None => Err(err),
            },
        }
    }

    // Returns the slot before the first unknown block on NEAR
    // The search range is [slot .. max_slot)
    // If there is no unknown block in this range max_slot - 1 will be returned
    // Assumptions:
    //     (1) block for slot is submitted to NEAR
    //     (2) block for max_slot is not submitted to NEAR
    fn linear_search_forward(
        &self,
        slot: u64,
        max_slot: u64,
        beacon_rpc_client: &BeaconRPCClient,
        eth_client_contract: &Box<dyn EthClientContractTrait>,
    ) -> u64 {
        let mut slot = slot;
        while slot < max_slot {
            match self.block_known_on_near(slot + 1, beacon_rpc_client, eth_client_contract) {
                Ok(true) => slot += 1,
                Ok(false) => break,
                Err(_) => slot += 1,
            }
        }

        slot
    }

    // Returns the slot before the first unknown block on NEAR
    // The search range is [last_slot .. start_slot)
    // If no such block are found the start_slot will be returned
    // Assumptions:
    //     (1) block for start_slot is submitted to NEAR
    //     (2) block for last_slot + 1 is not submitted to NEAR
    fn linear_search_backward(
        &self,
        start_slot: u64,
        last_slot: u64,
        beacon_rpc_client: &BeaconRPCClient,
        eth_client_contract: &Box<dyn EthClientContractTrait>,
    ) -> u64 {
        let mut slot = last_slot;
        let mut last_false_slot = slot + 1;

        while slot > start_slot {
            match self.block_known_on_near(slot, beacon_rpc_client, eth_client_contract) {
                Ok(true) => break,
                Ok(false) => {
                    last_false_slot = slot;
                    slot -= 1
                }
                Err(_) => slot -= 1,
            }
        }

        last_false_slot - 1
    }

    // Find the leftmost non-empty slot. Search range: [left_slot, right_slot).
    // Returns pair: (1) slot_id and (2) is this block already known on Eth client on NEAR
    // Assume that right_slot is non-empty and it's block were submitted to NEAR,
    // so if non correspondent block is found we return (right_slot, false)
    fn find_left_non_error_slot(
        &self,
        left_slot: u64,
        right_slot: u64,
        step: i8,
        beacon_rpc_client: &BeaconRPCClient,
        eth_client_contract: &Box<dyn EthClientContractTrait>,
    ) -> (u64, bool) {
        let mut slot = left_slot;
        while slot != right_slot {
            match self.block_known_on_near(slot, beacon_rpc_client, eth_client_contract) {
                Ok(v) => return (slot, v),
                Err(_) => {
                    if step > 0 {
                        slot += 1;
                    } else {
                        slot -= 1;
                    }
                }
            };
        }

        if step > 0 {
            (slot, false)
        } else {
            (slot, true)
        }
    }

    // Check if the block for current slot in Eth2 already were submitted to NEAR
    // Returns Error if slot doesn't contain any block
    fn block_known_on_near(
        &self,
        slot: u64,
        beacon_rpc_client: &BeaconRPCClient,
        eth_client_contract: &Box<dyn EthClientContractTrait>,
    ) -> Result<bool, Box<dyn Error>> {
        trace!(target: "relay", "Check if block with slot={} on NEAR", slot);
        match beacon_rpc_client.get_beacon_block_body_for_block_id(&format!("{}", slot)) {
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

                if eth_client_contract.is_known_block(&hash)? {
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

#[cfg(test)]
mod tests {
    use crate::beacon_rpc_client::BeaconRPCClient;
    use crate::config_for_tests::ConfigForTests;
    use crate::eth1_rpc_client::Eth1RPCClient;
    use crate::last_slot_searcher::LastSlotSearcher;
    use crate::test_utils::get_client_contract;
    use contract_wrapper::eth_client_contract_trait::EthClientContractTrait;
    use eth_types::BlockHeader;
    use std::error::Error;

    const TIMEOUT_SECONDS: u64 = 30;
    const TIMEOUT_STATE_SECONDS: u64 = 1000;

    fn get_test_config() -> ConfigForTests {
        ConfigForTests::load_from_toml("config_for_tests.toml".try_into().unwrap())
    }

    fn get_execution_block_by_slot(
        slot: u64,
        beacon_rpc_client: &BeaconRPCClient,
        eth1_rpc_client: &Eth1RPCClient,
    ) -> Result<BlockHeader, Box<dyn Error>> {
        match beacon_rpc_client.get_block_number_for_slot(types::Slot::new(slot)) {
            Ok(block_number) => eth1_rpc_client.get_block_header_by_number(block_number),
            Err(err) => Err(err),
        }
    }

    fn send_execution_blocks(
        beacon_rpc_client: &BeaconRPCClient,
        eth_client_contract: &mut Box<dyn EthClientContractTrait>,
        eth1_rpc_client: &Eth1RPCClient,
        start_slot: u64,
        end_slot: u64,
    ) {
        let mut slot = start_slot;
        let mut blocks: Vec<BlockHeader> = vec![];
        while slot <= end_slot {
            if let Ok(block) = get_execution_block_by_slot(slot, beacon_rpc_client, eth1_rpc_client)
            {
                blocks.push(block)
            }
            slot += 1;
        }

        eth_client_contract.send_headers(&blocks, end_slot).unwrap();
    }

    #[test]
    fn test_block_known_on_near() {
        let config_for_test = get_test_config();

        let mut eth_client_contract = get_client_contract(true, &config_for_test);
        eth_client_contract.register_submitter().unwrap();
        let beacon_rpc_client = BeaconRPCClient::new(
            &config_for_test.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let eth1_rpc_client = Eth1RPCClient::new(&config_for_test.eth1_endpoint);
        let last_slot_searcher = LastSlotSearcher::new(true);

        let is_block_known = last_slot_searcher.block_known_on_near(
            config_for_test.slot_without_block,
            &beacon_rpc_client,
            &eth_client_contract,
        );
        if is_block_known.is_ok() {
            panic!();
        }

        let is_block_known = last_slot_searcher.block_known_on_near(
            config_for_test.first_slot,
            &beacon_rpc_client,
            &eth_client_contract,
        );

        match is_block_known {
            Ok(is_block_known) => assert!(!is_block_known),
            Err(_) => panic!(),
        }

        let finalized_slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            finalized_slot + 1,
            finalized_slot + 1,
        );

        let is_block_known = last_slot_searcher.block_known_on_near(
            finalized_slot + 1,
            &beacon_rpc_client,
            &eth_client_contract,
        );
        match is_block_known {
            Ok(is_block_known) => assert!(is_block_known),
            Err(_) => panic!(),
        }
    }

    #[test]
    fn test_find_left_non_error_slot() {
        let config_for_test = get_test_config();
        let mut eth_client_contract = get_client_contract(true, &config_for_test);
        eth_client_contract.register_submitter().unwrap();
        let beacon_rpc_client = BeaconRPCClient::new(
            &config_for_test.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let eth1_rpc_client = Eth1RPCClient::new(&config_for_test.eth1_endpoint);
        let last_slot_searcher = LastSlotSearcher::new(true);

        let (left_non_empty_slot, is_known_block) = last_slot_searcher.find_left_non_error_slot(
            config_for_test.left_empty_slot - 1,
            config_for_test.right_empty_slot + 2,
            1,
            &beacon_rpc_client,
            &eth_client_contract,
        );
        assert_eq!(left_non_empty_slot, config_for_test.left_empty_slot - 1);
        assert!(!is_known_block);

        let (left_non_empty_slot, is_known_block) = last_slot_searcher.find_left_non_error_slot(
            config_for_test.left_empty_slot,
            config_for_test.right_empty_slot + 2,
            1,
            &beacon_rpc_client,
            &eth_client_contract,
        );
        assert_eq!(left_non_empty_slot, config_for_test.right_empty_slot + 1);
        assert!(!is_known_block);

        let (left_non_empty_slot, is_known_block) = last_slot_searcher.find_left_non_error_slot(
            config_for_test.left_empty_slot,
            config_for_test.right_empty_slot,
            1,
            &beacon_rpc_client,
            &eth_client_contract,
        );
        assert_eq!(left_non_empty_slot, config_for_test.right_empty_slot);
        assert!(!is_known_block);

        let (left_non_empty_slot, is_known_block) = last_slot_searcher.find_left_non_error_slot(
            config_for_test.right_empty_slot,
            config_for_test.right_empty_slot + 2,
            1,
            &beacon_rpc_client,
            &eth_client_contract,
        );
        assert_eq!(left_non_empty_slot, config_for_test.right_empty_slot + 1);
        assert!(!is_known_block);

        let finalized_slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            finalized_slot + 1,
            finalized_slot + 1,
        );

        let (left_non_empty_slot, is_known_block) = last_slot_searcher.find_left_non_error_slot(
            finalized_slot + 1,
            finalized_slot + 2,
            1,
            &beacon_rpc_client,
            &eth_client_contract,
        );
        assert_eq!(left_non_empty_slot, finalized_slot + 1);
        assert!(is_known_block);
    }

    #[test]
    fn test_linear_search_backward() {
        let config_for_test = get_test_config();
        let mut eth_client_contract = get_client_contract(true, &config_for_test);
        eth_client_contract.register_submitter().unwrap();
        let beacon_rpc_client = BeaconRPCClient::new(
            &config_for_test.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let eth1_rpc_client = Eth1RPCClient::new(&config_for_test.eth1_endpoint);
        let last_slot_searcher = LastSlotSearcher::new(true);

        let finalized_slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();
        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            finalized_slot + 1,
            finalized_slot + 2,
        );

        let last_submitted_block = last_slot_searcher.linear_search_backward(
            finalized_slot + 1,
            finalized_slot + 10,
            &beacon_rpc_client,
            &eth_client_contract,
        );
        assert_eq!(last_submitted_block, finalized_slot + 2);

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            finalized_slot + 3,
            config_for_test.slot_without_block - 1,
        );

        let last_submitted_block = last_slot_searcher.linear_search_backward(
            finalized_slot + 1,
            config_for_test.right_bound_in_slot_search,
            &beacon_rpc_client,
            &eth_client_contract,
        );
        assert_eq!(last_submitted_block, config_for_test.slot_without_block);
    }

    #[test]
    fn test_linear_search_forward() {
        let config_for_test = get_test_config();
        let mut eth_client_contract = get_client_contract(true, &config_for_test);
        eth_client_contract.register_submitter().unwrap();
        let beacon_rpc_client = BeaconRPCClient::new(
            &config_for_test.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let eth1_rpc_client = Eth1RPCClient::new(&config_for_test.eth1_endpoint);
        let last_slot_searcher = LastSlotSearcher::new(true);

        let mut slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();
        slot += 1;

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            slot,
            config_for_test.slot_without_block - 2,
        );

        let last_block_on_near = last_slot_searcher.linear_search_forward(
            eth_client_contract
                .get_finalized_beacon_block_slot()
                .unwrap()
                + 1,
            config_for_test.right_bound_in_slot_search,
            &beacon_rpc_client,
            &eth_client_contract,
        );

        assert_eq!(last_block_on_near, config_for_test.slot_without_block - 2);

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            config_for_test.slot_without_block - 1,
            config_for_test.slot_without_block - 1,
        );

        let last_block_on_near = last_slot_searcher.linear_search_forward(
            eth_client_contract
                .get_finalized_beacon_block_slot()
                .unwrap()
                + 1,
            config_for_test.right_bound_in_slot_search,
            &beacon_rpc_client,
            &eth_client_contract,
        );

        assert_eq!(last_block_on_near, config_for_test.slot_without_block);
    }

    #[test]
    fn test_linear_slot_search() {
        let config_for_test = get_test_config();
        let mut eth_client_contract = get_client_contract(true, &config_for_test);
        eth_client_contract.register_submitter().unwrap();
        let beacon_rpc_client = BeaconRPCClient::new(
            &config_for_test.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let eth1_rpc_client = Eth1RPCClient::new(&config_for_test.eth1_endpoint);
        let last_slot_searcher = LastSlotSearcher::new(true);

        let mut slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();
        slot += 1;
        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            slot,
            config_for_test.slot_without_block - 1,
        );

        let finalized_slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();

        let last_block_on_near = last_slot_searcher
            .linear_slot_search(
                config_for_test.slot_without_block - 1,
                finalized_slot,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);

        let last_block_on_near = last_slot_searcher
            .linear_slot_search(
                config_for_test.slot_without_block,
                finalized_slot,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);

        let last_block_on_near = last_slot_searcher
            .linear_slot_search(
                config_for_test.first_slot + 1,
                finalized_slot,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);

        let last_block_on_near = last_slot_searcher
            .linear_slot_search(
                config_for_test.slot_without_block + 5,
                finalized_slot,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);
    }

    #[test]
    #[should_panic]
    fn test_error_on_connection_problem() {
        let config_for_test = get_test_config();
        let mut eth_client_contract = get_client_contract(true, &config_for_test);
        eth_client_contract.register_submitter().unwrap();
        let mut beacon_rpc_client = BeaconRPCClient::new(
            &config_for_test.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let eth1_rpc_client = Eth1RPCClient::new(&config_for_test.eth1_endpoint);
        let last_slot_searcher = LastSlotSearcher::new(true);

        let finalized_slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            finalized_slot + 1,
            finalized_slot + 2,
        );

        beacon_rpc_client = BeaconRPCClient::new(
            "http://httpstat.us/504/",
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        last_slot_searcher
            .linear_slot_search(
                finalized_slot + 1,
                finalized_slot,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
    }

    #[test]
    fn test_binsearch_slot_range() {
        let config_for_test = get_test_config();
        let mut eth_client_contract = get_client_contract(true, &config_for_test);
        eth_client_contract.register_submitter().unwrap();
        let mut beacon_rpc_client = BeaconRPCClient::new(
            &config_for_test.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let eth1_rpc_client = Eth1RPCClient::new(&config_for_test.eth1_endpoint);
        let last_slot_searcher = LastSlotSearcher::new(true);

        let finalized_beacon_slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            finalized_beacon_slot + 1,
            config_for_test.slot_without_block - 2,
        );

        let last_block_on_near = last_slot_searcher
            .binsearch_slot_range(
                eth_client_contract
                    .get_finalized_beacon_block_slot()
                    .unwrap()
                    + 1,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block - 2);

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            config_for_test.slot_without_block - 1,
            config_for_test.slot_without_block - 1,
        );
        let last_block_on_near = last_slot_searcher
            .binsearch_slot_range(
                eth_client_contract
                    .get_finalized_beacon_block_slot()
                    .unwrap()
                    + 1,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);

        let last_block_on_near = last_slot_searcher
            .binsearch_slot_range(
                eth_client_contract
                    .get_finalized_beacon_block_slot()
                    .unwrap()
                    + 1,
                config_for_test.slot_without_block,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block - 1);

        let last_block_on_near = last_slot_searcher
            .binsearch_slot_range(
                config_for_test.slot_without_block,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);

        beacon_rpc_client = BeaconRPCClient::new(
            "http://httpstat.us/504/",
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        if last_slot_searcher
            .binsearch_slot_range(
                eth_client_contract
                    .get_finalized_beacon_block_slot()
                    .unwrap()
                    + 1,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .is_ok()
        {
            panic!("binarysearch returns result in unworking network");
        }
    }

    #[test]
    fn test_binsearch_slot_forward() {
        let config_for_test = get_test_config();
        let mut eth_client_contract = get_client_contract(true, &config_for_test);
        eth_client_contract.register_submitter().unwrap();
        let mut beacon_rpc_client = BeaconRPCClient::new(
            &config_for_test.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let eth1_rpc_client = Eth1RPCClient::new(&config_for_test.eth1_endpoint);
        let last_slot_searcher = LastSlotSearcher::new(true);

        let finalized_beacon_slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            finalized_beacon_slot + 1,
            config_for_test.slot_without_block - 2,
        );

        let last_block_on_near = last_slot_searcher
            .binsearch_slot_forward(
                eth_client_contract
                    .get_finalized_beacon_block_slot()
                    .unwrap()
                    + 1,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block - 2);

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            config_for_test.slot_without_block - 1,
            config_for_test.slot_without_block - 1,
        );

        let last_block_on_near = last_slot_searcher
            .binsearch_slot_forward(
                eth_client_contract
                    .get_finalized_beacon_block_slot()
                    .unwrap()
                    + 1,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);

        let last_block_on_near = last_slot_searcher
            .binsearch_slot_forward(
                eth_client_contract
                    .get_finalized_beacon_block_slot()
                    .unwrap()
                    + 1,
                config_for_test.slot_without_block,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block - 1);

        let last_block_on_near = last_slot_searcher
            .binsearch_slot_forward(
                config_for_test.slot_without_block,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);

        beacon_rpc_client = BeaconRPCClient::new(
            "http://httpstat.us/504/",
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        if last_slot_searcher
            .binsearch_slot_forward(
                eth_client_contract
                    .get_finalized_beacon_block_slot()
                    .unwrap()
                    + 1,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .is_ok()
        {
            panic!("binarysearch returns result in unworking network");
        }
    }

    #[test]
    fn test_binsearch_slot_search() {
        let config_for_test = get_test_config();
        let mut eth_client_contract = get_client_contract(true, &config_for_test);
        eth_client_contract.register_submitter().unwrap();
        let mut beacon_rpc_client = BeaconRPCClient::new(
            &config_for_test.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let eth1_rpc_client = Eth1RPCClient::new(&config_for_test.eth1_endpoint);
        let last_slot_searcher = LastSlotSearcher::new(true);

        let finalized_slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            finalized_slot + 1,
            config_for_test.slot_without_block - 2,
        );

        let finalized_slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();

        let last_block_on_near = last_slot_searcher
            .binary_slot_search(
                finalized_slot + 1,
                finalized_slot,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block - 2);

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            config_for_test.slot_without_block - 1,
            config_for_test.slot_without_block - 1,
        );

        let last_block_on_near = last_slot_searcher
            .binary_slot_search(
                finalized_slot + 1,
                finalized_slot,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);

        let last_block_on_near = last_slot_searcher
            .binary_slot_search(
                finalized_slot + 1,
                finalized_slot,
                config_for_test.slot_without_block,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);

        let last_block_on_near = last_slot_searcher
            .binary_slot_search(
                finalized_slot + 1,
                finalized_slot,
                config_for_test.slot_without_block - 1,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block - 1);

        let last_block_on_near = last_slot_searcher
            .binary_slot_search(
                config_for_test.slot_without_block,
                finalized_slot,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block - 1);

        beacon_rpc_client = BeaconRPCClient::new(
            "http://httpstat.us/504/",
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        if last_slot_searcher
            .binary_slot_search(
                finalized_slot + 1,
                finalized_slot,
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .is_ok()
        {
            panic!("binarysearch returns result in unworking network");
        }
    }

    #[test]
    fn test_get_last_slot_binsearch() {
        let config_for_test = get_test_config();
        let mut eth_client_contract = get_client_contract(true, &config_for_test);
        eth_client_contract.register_submitter().unwrap();
        let mut beacon_rpc_client = BeaconRPCClient::new(
            &config_for_test.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let eth1_rpc_client = Eth1RPCClient::new(&config_for_test.eth1_endpoint);
        let mut last_slot_searcher = LastSlotSearcher::new(true);

        let finalized_slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();
        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            finalized_slot + 1,
            config_for_test.slot_without_block - 2,
        );

        let last_block_on_near = last_slot_searcher
            .get_last_slot(
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block - 2);

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            config_for_test.slot_without_block - 1,
            config_for_test.slot_without_block - 1,
        );

        let last_block_on_near = last_slot_searcher
            .get_last_slot(
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);

        beacon_rpc_client = BeaconRPCClient::new(
            "http://httpstat.us/504/",
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        if last_slot_searcher
            .get_last_slot(
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .is_ok()
        {
            panic!("binarysearch returns result in unworking network");
        }
    }

    #[test]
    fn test_get_last_slot_linearsearch() {
        let config_for_test = get_test_config();
        let mut eth_client_contract = get_client_contract(true, &config_for_test);
        eth_client_contract.register_submitter().unwrap();
        let mut beacon_rpc_client = BeaconRPCClient::new(
            &config_for_test.beacon_endpoint,
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        let eth1_rpc_client = Eth1RPCClient::new(&config_for_test.eth1_endpoint);
        let mut last_slot_searcher = LastSlotSearcher::new(true);

        let finalized_slot = eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap();

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            finalized_slot + 1,
            config_for_test.slot_without_block - 2,
        );

        let last_block_on_near = last_slot_searcher
            .get_last_slot(
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block - 2);

        send_execution_blocks(
            &beacon_rpc_client,
            &mut eth_client_contract,
            &eth1_rpc_client,
            config_for_test.slot_without_block - 1,
            config_for_test.slot_without_block - 1,
        );

        let last_block_on_near = last_slot_searcher
            .get_last_slot(
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_block_on_near, config_for_test.slot_without_block);

        beacon_rpc_client = BeaconRPCClient::new(
            "http://httpstat.us/504/",
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        if last_slot_searcher
            .get_last_slot(
                config_for_test.right_bound_in_slot_search,
                &beacon_rpc_client,
                &eth_client_contract,
            )
            .is_ok()
        {
            panic!("binarysearch returns result in unworking network");
        }
    }
}
