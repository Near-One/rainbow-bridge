use crate::beacon_rpc_client::BeaconRPCClient;
use crate::config::Config;
use crate::eth1_rpc_client::Eth1RPCClient;
use crate::hand_made_finality_light_client_update::HandMadeFinalityLightClientUpdate;
use crate::last_slot_searcher::LastSlotSearcher;
use crate::near_rpc_client::NearRPCClient;
use crate::prometheus_metrics;
use crate::prometheus_metrics::{
    FAILS_ON_HEADERS_SUBMISSION, FAILS_ON_UPDATES_SUBMISSION, LAST_ETH_SLOT, LAST_ETH_SLOT_ON_NEAR,
    LAST_FINALIZED_ETH_SLOT, LAST_FINALIZED_ETH_SLOT_ON_NEAR,
};
use crate::relay_errors::NoBlockForSlotError;
use contract_wrapper::eth_client_contract_trait::EthClientContractTrait;
use eth_types::eth2::LightClientUpdate;
use eth_types::BlockHeader;
use log::{debug, info, trace, warn};
use near_primitives::views::FinalExecutionStatus;
use std::error::Error;
use std::thread::sleep;
use std::thread::spawn;
use std::time::Duration;
use std::vec::Vec;

const ONE_EPOCH_IN_SLOTS: u64 = 32;

macro_rules! skip_fail {
    ($res:expr, $msg:expr, $sleep_time:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                warn!(target: "relay", "{}. Error: {}", $msg, e);
                trace!(target: "relay", "Sleep {} secs before next loop", $sleep_time);
                sleep(Duration::from_secs($sleep_time));
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

macro_rules! return_on_fail_and_sleep {
    ($res:expr, $msg:expr, $sleep_time:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                warn!(target: "relay", "{}. Error: {}", $msg, e);
                trace!(target: "relay", "Sleep {} secs before next loop", $sleep_time);
                sleep(Duration::from_secs($sleep_time));
                return;
            }
        }
    };
}

pub struct Eth2NearRelay {
    beacon_rpc_client: BeaconRPCClient,
    eth1_rpc_client: Eth1RPCClient,
    near_rpc_client: NearRPCClient,
    eth_client_contract: Box<dyn EthClientContractTrait>,
    max_submitted_headers: u64,
    network: String,
    light_client_updates_submission_frequency_in_epochs: u64,
    max_blocks_for_finalization: u64,
    near_network_name: String,
    last_slot_searcher: LastSlotSearcher,
    terminate: bool,
    submit_only_finalized_blocks: bool,
    next_light_client_update: Option<LightClientUpdate>,
    sleep_time_on_sync_secs: u64,
    sleep_time_after_submission_secs: u64,
}

impl Eth2NearRelay {
    pub fn init(
        config: &Config,
        eth_contract: Box<dyn EthClientContractTrait>,
        enable_binsearch: bool,
        submit_only_finalized_blocks: bool,
    ) -> Self {
        info!(target: "relay", "=== Relay initialization === ");

        let beacon_rpc_client = BeaconRPCClient::new(
            &config.beacon_endpoint,
            config.eth_requests_timeout_seconds,
            config.state_requests_timeout_seconds,
        );
        let next_light_client_update =
            Self::get_light_client_update_from_file(config, &beacon_rpc_client).unwrap();

        let eth2near_relay = Eth2NearRelay {
            beacon_rpc_client,
            eth1_rpc_client: Eth1RPCClient::new(&config.eth1_endpoint),
            eth_client_contract: eth_contract,
            near_rpc_client: NearRPCClient::new(&config.near_endpoint),
            max_submitted_headers: config.total_submit_headers as u64,
            network: config.network.to_string(),
            light_client_updates_submission_frequency_in_epochs: config
                .light_client_updates_submission_frequency_in_epochs,
            max_blocks_for_finalization: config.max_blocks_for_finalization,
            near_network_name: config.near_network_id.to_string(),
            last_slot_searcher: LastSlotSearcher::new(enable_binsearch),
            terminate: false,
            submit_only_finalized_blocks,
            next_light_client_update,
            sleep_time_on_sync_secs: config.sleep_time_on_sync_secs,
            sleep_time_after_submission_secs: config.sleep_time_after_submission_secs,
        };

        if !eth2near_relay
            .eth_client_contract
            .is_submitter_registered(None)
            .unwrap_or_else(|e| panic!("Failed to check if the submitter registered. Err {}", e))
        {
            eth2near_relay
                .eth_client_contract
                .register_submitter()
                .unwrap();
        }

        if let Some(port) = config.prometheus_metrics_port {
            spawn(move || prometheus_metrics::run_prometheus_service(port));
        }

        eth2near_relay
    }

    pub fn run(&mut self, max_iterations: Option<u64>) {
        info!(target: "relay", "=== Relay running ===");
        let mut iter_id = 0;
        while !self.terminate {
            let mut were_submission_on_iter: bool = false;
            iter_id += 1;
            self.set_terminate(iter_id, max_iterations);
            skip_fail!(
                self.wait_for_synchronization(),
                "Fail to get sync status",
                self.sleep_time_on_sync_secs
            );

            info!(target: "relay", "== New relay loop ==");

            let last_eth2_slot_on_eth_chain: u64 = if self.submit_only_finalized_blocks {
                skip_fail!(
                    self.beacon_rpc_client.get_last_finalized_slot_number(),
                    "Fail to get last finalized slot on Eth",
                    self.sleep_time_on_sync_secs
                )
                .as_u64()
            } else {
                skip_fail!(
                    self.beacon_rpc_client.get_last_slot_number(),
                    "Fail to get last slot on Eth",
                    self.sleep_time_on_sync_secs
                )
                .as_u64()
            };
            let mut last_eth2_slot_on_near: u64 = skip_fail!(
                self.last_slot_searcher.get_last_slot(
                    last_eth2_slot_on_eth_chain,
                    &self.beacon_rpc_client,
                    &self.eth_client_contract
                ),
                "Fail to get last slot on NEAR",
                self.sleep_time_on_sync_secs
            );

            LAST_ETH_SLOT.inc_by(last_eth2_slot_on_eth_chain as i64 - LAST_ETH_SLOT.get());
            LAST_ETH_SLOT_ON_NEAR
                .inc_by(last_eth2_slot_on_near as i64 - LAST_ETH_SLOT_ON_NEAR.get());

            info!(target: "relay", "Last slot on near = {}; last slot on eth = {}",
                  last_eth2_slot_on_near, last_eth2_slot_on_eth_chain);

            if last_eth2_slot_on_near < last_eth2_slot_on_eth_chain {
                info!(target: "relay", "= Creating headers batch =");

                let (headers, current_slot) = skip_fail!(
                    self.get_execution_blocks_between(
                        last_eth2_slot_on_near + 1,
                        last_eth2_slot_on_eth_chain,
                    ),
                    "Network problems during fetching execution blocks",
                    self.sleep_time_on_sync_secs
                );
                self.submit_execution_blocks(headers, current_slot, &mut last_eth2_slot_on_near);
                were_submission_on_iter = true;
            }

            let last_finalized_slot_on_near: u64 = skip_fail!(
                self.eth_client_contract.get_finalized_beacon_block_slot(),
                "Error on getting finalized block hash. Skipping sending light client update",
                self.sleep_time_on_sync_secs);
            
            let last_finalized_slot_on_eth: u64 = skip_fail!(self
                .beacon_rpc_client
                .get_last_finalized_slot_number(),
                "Error on getting last finalized slot number on Ethereum. Skipping sending light client update",
                self.sleep_time_on_sync_secs).as_u64();

            LAST_FINALIZED_ETH_SLOT_ON_NEAR
                .inc_by(last_finalized_slot_on_near as i64 - LAST_FINALIZED_ETH_SLOT_ON_NEAR.get());
            LAST_FINALIZED_ETH_SLOT
                .inc_by(last_finalized_slot_on_eth as i64 - LAST_FINALIZED_ETH_SLOT.get());

            trace!(target: "relay", "last_finalized_slot on near/eth {}/{}", last_finalized_slot_on_near, last_finalized_slot_on_eth);

            if self.is_enough_blocks_for_light_client_update(
                last_eth2_slot_on_near,
                last_finalized_slot_on_near,
                last_finalized_slot_on_eth,
            ) {
                self.send_light_client_updates(
                    last_eth2_slot_on_near,
                    last_finalized_slot_on_near,
                    last_finalized_slot_on_eth,
                );
                were_submission_on_iter = true;
            }

            if !were_submission_on_iter {
                info!(target: "relay", "Sync with ETH network. Sleep {} secs", self.sleep_time_on_sync_secs);
                sleep(Duration::from_secs(self.sleep_time_on_sync_secs));
            }
        }
    }

    fn wait_for_synchronization(&self) -> Result<(), Box<dyn Error>> {
        while self.beacon_rpc_client.is_syncing()?
            || self.eth1_rpc_client.is_syncing()?
            || self.near_rpc_client.is_syncing()?
        {
            info!(target: "relay", "Waiting for sync...");
            sleep(Duration::from_secs(self.sleep_time_on_sync_secs));
        }
        Ok(())
    }

    fn get_light_client_update_from_file(
        config: &Config,
        beacon_rpc_client: &BeaconRPCClient,
    ) -> Result<Option<LightClientUpdate>, Box<dyn Error>> {
        let mut next_light_client_update: Option<LightClientUpdate> = None;
        if let Some(path_to_attested_state) = config.clone().path_to_attested_state {
            match config.clone().path_to_finality_state {
                Some(path_to_finality_state) => {
                    next_light_client_update = Some(
                        HandMadeFinalityLightClientUpdate::get_light_client_update_from_file_with_next_sync_committee(
                            beacon_rpc_client,
                            &path_to_attested_state,
                            &path_to_finality_state,
                        ).unwrap(),
                    );
                }
                None => {
                    next_light_client_update = Some(
                        HandMadeFinalityLightClientUpdate::get_finality_light_client_update_from_file(
                            beacon_rpc_client,
                            &path_to_attested_state,
                        ).unwrap(),
                    );
                }
            }
        }
        Ok(next_light_client_update)
    }

    fn set_terminate(&mut self, iter_id: u64, max_iterations: Option<u64>) {
        if let Some(max_iter) = max_iterations {
            if iter_id > max_iter {
                self.terminate = true;
            }
        }
    }

    fn get_execution_blocks_between(
        &self,
        start_slot: u64,
        last_eth2_slot_on_eth_chain: u64,
    ) -> Result<(Vec<BlockHeader>, u64), Box<dyn Error>> {
        let mut headers: Vec<BlockHeader> = vec![];
        let mut current_slot = start_slot;

        while headers.len() < self.max_submitted_headers as usize
            && current_slot <= last_eth2_slot_on_eth_chain
        {
            debug!(target: "relay", "Try add block header for slot={}, headers len={}/{}", current_slot, headers.len(), self.max_submitted_headers);
            match self.get_execution_block_by_slot(current_slot) {
                Ok(eth1_header) => headers.push(eth1_header),
                Err(err) => match err.downcast_ref::<NoBlockForSlotError>() {
                    Some(_) => {
                        current_slot += 1;
                        continue;
                    }
                    None => return Err(err),
                },
            }
            current_slot += 1;
        }

        Ok((headers, current_slot))
    }

    fn submit_execution_blocks(
        &mut self,
        headers: Vec<BlockHeader>,
        current_slot: u64,
        last_eth2_slot_on_near: &mut u64,
    ) {
        info!(target: "relay", "Try submit headers from slot={} to {} to NEAR", *last_eth2_slot_on_near + 1, current_slot - 1);
        let execution_outcome = return_on_fail!(
            self.eth_client_contract
                .send_headers(&headers, current_slot - 1),
            "Error on header submission"
        );

        if let FinalExecutionStatus::Failure(error_message) = execution_outcome.status {
            FAILS_ON_HEADERS_SUBMISSION.inc();
            warn!(target: "relay", "FAIL status on Headers submission. Error: {:?}", error_message);
        }

        *last_eth2_slot_on_near = current_slot - 1;
        info!(target: "relay", "Successful headers submission! Transaction URL: https://explorer.{}.near.org/transactions/{}",
                                  self.near_network_name, execution_outcome.transaction.hash);
        sleep(Duration::from_secs(self.sleep_time_after_submission_secs));
    }

    fn verify_bls_signature_for_finality_update(
        &mut self,
        light_client_update: &LightClientUpdate,
    ) -> Result<bool, Box<dyn Error>> {
        let signature_slot_period =
            BeaconRPCClient::get_period_for_slot(light_client_update.signature_slot);
        let finalized_slot_period = BeaconRPCClient::get_period_for_slot(
            self.eth_client_contract.get_finalized_beacon_block_slot()?,
        );

        let light_client_state = self.eth_client_contract.get_light_client_state()?;

        let sync_committee = if signature_slot_period == finalized_slot_period {
            light_client_state.current_sync_committee
        } else {
            light_client_state.next_sync_committee
        };

        finality_update_verify::is_correct_finality_update(
            &self.network,
            light_client_update,
            sync_committee,
        )
    }

    fn get_execution_block_by_slot(&self, slot: u64) -> Result<BlockHeader, Box<dyn Error>> {
        match self
            .beacon_rpc_client
            .get_block_number_for_slot(types::Slot::new(slot))
        {
            Ok(block_number) => self
                .eth1_rpc_client
                .get_block_header_by_number(block_number),
            Err(err) => Err(err),
        }
    }
}

// Implementation of functions for submitting light client updates
impl Eth2NearRelay {
    fn is_enough_blocks_for_light_client_update(
        &self,
        last_submitted_slot: u64,
        last_finalized_slot_on_near: u64,
        last_finalized_slot_on_eth: u64,
    ) -> bool {
        if (last_submitted_slot as i64) - (last_finalized_slot_on_near as i64)
            < (ONE_EPOCH_IN_SLOTS * self.light_client_updates_submission_frequency_in_epochs) as i64
        {
            info!(target: "relay", "Light client update were send less then {} epochs ago. Skipping sending light client update", self.light_client_updates_submission_frequency_in_epochs);
            return false;
        }

        if last_finalized_slot_on_eth <= last_finalized_slot_on_near {
            info!(target: "relay", "Last finalized slot on Eth equal to last finalized slot on NEAR. Skipping sending light client update.");
            return false;
        }

        true
    }

    fn is_shot_run_mode(&self) -> bool {
        self.next_light_client_update.is_some()
    }

    fn send_light_client_updates(
        &mut self,
        last_submitted_slot: u64,
        last_finalized_slot_on_near: u64,
        last_finalized_slot_on_eth: u64,
    ) {
        info!(target: "relay", "= Sending light client update =");

        if self.is_shot_run_mode() {
            info!(target: "relay", "Try sending light client update from file");
            self.send_light_client_update_from_file(last_submitted_slot);
            return;
        }

        if last_finalized_slot_on_eth
            >= last_finalized_slot_on_near + self.max_blocks_for_finalization
        {
            info!(target: "relay", "Too big gap between slot of finalized block on Near and Eth. Sending hand made light client update");
            self.send_hand_made_light_client_update(last_finalized_slot_on_near);
        } else {
            self.send_regular_light_client_update(
                last_finalized_slot_on_eth,
                last_finalized_slot_on_near,
            );
        }
    }

    fn send_light_client_update_from_file(&mut self, last_submitted_slot: u64) {
        if let Some(light_client_update) = self.next_light_client_update.clone() {
            if last_submitted_slot < light_client_update.attested_beacon_header.slot {
                return;
            }

            self.send_specific_light_cleint_update(light_client_update);
            self.terminate = true;
        }
    }

    fn send_regular_light_client_update(
        &mut self,
        last_finalized_slot_on_eth: u64,
        last_finalized_slot_on_near: u64,
    ) {
        let last_eth2_period_on_near_chain =
            BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_near);
        info!(target: "relay", "Last finalized slot/period on near={}/{}", last_finalized_slot_on_near, last_eth2_period_on_near_chain);

        let end_period = BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_eth);
        info!(target: "relay", "Last finalized slot/period on ethereum={}/{}", last_finalized_slot_on_eth, end_period);

        let light_client_update = if end_period == last_eth2_period_on_near_chain {
            debug!(target: "relay", "Finalized period on Eth and Near are equal. Don't fetch sync commity update");
            return_on_fail!(
                self.beacon_rpc_client.get_finality_light_client_update(),
                "Error on getting light client update. Skipping sending light client update"
            )
        } else {
            debug!(target: "relay", "Finalized period on Eth and Near are different. Fetching sync commity update");
            return_on_fail!(
                self.beacon_rpc_client
                    .get_finality_light_client_update_with_sync_commity_update(),
                "Error on getting light client update. Skipping sending light client update"
            )
        };

        self.send_specific_light_cleint_update(light_client_update);
    }

    fn get_attested_slot(
        &mut self,
        last_finalized_slot_on_near: u64,
    ) -> Result<u64, Box<dyn Error>> {
        const EXPECTED_EPOCHS_BETWEEN_HEAD_AND_FINALIZED_BLOCKS: u64 = 2;
        let next_finalized_slot = last_finalized_slot_on_near
            + self.light_client_updates_submission_frequency_in_epochs * ONE_EPOCH_IN_SLOTS;
        let attested_slot = next_finalized_slot
            + EXPECTED_EPOCHS_BETWEEN_HEAD_AND_FINALIZED_BLOCKS * ONE_EPOCH_IN_SLOTS;

        let attested_slot: u64 = self
            .beacon_rpc_client
            .get_non_empty_beacon_block_header(attested_slot)?
            .slot
            .into();
        trace!(target: "relay", "Chosen attested slot {}", attested_slot);

        Ok(attested_slot)
    }

    fn send_hand_made_light_client_update(&mut self, last_finalized_slot_on_near: u64) {
        let mut attested_slot = return_on_fail!(
            self.get_attested_slot(last_finalized_slot_on_near),
            "Error on getting attested slot"
        );

        let include_next_sync_committee =
            BeaconRPCClient::get_period_for_slot(last_finalized_slot_on_near)
                != BeaconRPCClient::get_period_for_slot(attested_slot);

        loop {
            let light_client_update = return_on_fail!(
                HandMadeFinalityLightClientUpdate::get_finality_light_client_update(
                    &self.beacon_rpc_client,
                    attested_slot,
                    include_next_sync_committee,
                ),
                format!(
                    "Error on getting hand made light client update for attested slot={}.",
                    attested_slot
                )
            );

            let finality_update_slot = light_client_update
                .finality_update
                .header_update
                .beacon_header
                .slot;

            if finality_update_slot <= last_finalized_slot_on_near {
                info!(target: "relay", "Finality update slot for hand made light client update <= last finality update on near. Increment gap for attested slot and skipping light client update.");
                attested_slot = return_on_fail!(
                    self.get_attested_slot(last_finalized_slot_on_near + ONE_EPOCH_IN_SLOTS),
                    "Error on getting attested slot"
                );
                continue;
            }

            trace!(target: "relay", "Hand made light client update: {:?}", light_client_update);
            self.send_specific_light_cleint_update(light_client_update);
            return;
        }
    }

    fn send_specific_light_cleint_update(&mut self, light_client_update: LightClientUpdate) {
        let is_known_block = return_on_fail!(
            self.eth_client_contract.is_known_block(
                &light_client_update
                    .finality_update
                    .header_update
                    .execution_block_hash,
            ),
            "Fail on the is_known_block method. Skipping sending light client update"
        );

        if is_known_block {
            let verification_result = return_on_fail!(
                self.verify_bls_signature_for_finality_update(&light_client_update),
                "Error on bls verification. Skip sending the light client update"
            );

            if verification_result {
                info!(target: "relay", "PASS bls signature verification!");
            } else {
                warn!(target: "relay", "NOT PASS bls signature verification. Skip sending this light client update");
                return;
            }

            info!(target: "relay", "Sending light client update");

            let execution_outcome = return_on_fail_and_sleep!(
                self.eth_client_contract
                    .send_light_client_update(light_client_update),
                "Fail to send light client update",
                self.sleep_time_on_sync_secs
            );

            if let FinalExecutionStatus::Failure(error_message) = execution_outcome.status {
                FAILS_ON_UPDATES_SUBMISSION.inc();
                warn!(target: "relay", "FAIL status on Light Client Update submission. Error: {:?}", error_message);
            }

            info!(target: "relay", "Successful light client update submission! Transaction URL: https://explorer.{}.near.org/transactions/{}",
                                  self.near_network_name, execution_outcome.transaction.hash);
            sleep(Duration::from_secs(self.sleep_time_after_submission_secs));
        } else {
            debug!(target: "relay", "Finalized block for light client update is not found on NEAR. Skipping send light client update");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::beacon_rpc_client::BeaconRPCClient;
    use crate::eth2near_relay::{Eth2NearRelay, ONE_EPOCH_IN_SLOTS};
    use crate::hand_made_finality_light_client_update::HandMadeFinalityLightClientUpdate;
    use crate::relay_errors::NoBlockForSlotError;
    use crate::test_utils::{get_relay, get_relay_from_slot, get_relay_with_update_from_file};
    use eth_types::eth2::LightClientUpdate;
    use eth_types::BlockHeader;

    const FIRST_SLOT: u64 = 1099360;
    const RIGHT_BOUND_IN_SLOT_SEARCH: u64 = 1099500;
    const SLOT_WITHOUT_BLOCK: u64 = 1099364;
    const FINALIZED_SLOT_1: u64 = 1099392;
    const FINALIZED_SLOT_2: u64 = 1099488;
    const FINALIZED_SLOT_7: u64 = 1099808;
    const FINALIZED_SLOT_8: u64 = 1099872;
    const FINALIZED_SLOT_BEFORE_NEW_PERIOD: u64 = 1105919;
    const TIMEOUT_SECONDS: u64 = 30;
    const TIMEOUT_STATE_SECONDS: u64 = 1000;

    fn send_execution_blocks_between(relay: &mut Eth2NearRelay, start_slot: u64, end_slot: u64) {
        let mut slot = start_slot;
        let mut blocks: Vec<BlockHeader> = vec![];
        while slot <= end_slot {
            if let Ok(block) = relay.get_execution_block_by_slot(slot) {
                blocks.push(block)
            }
            slot += 1;
        }

        println!("Submitted blocks: {}", blocks.len());

        relay
            .eth_client_contract
            .send_headers(&blocks, end_slot)
            .unwrap();
    }

    fn send_blocks_till_finalized_eth_slot(relay: &mut Eth2NearRelay, finality_slot: u64) -> u64 {
        let mut slot = finality_slot + 1;

        let mut finality_slot_on_eth = relay
            .beacon_rpc_client
            .get_last_finalized_slot_number()
            .unwrap()
            .as_u64();

        let mut blocks: Vec<BlockHeader> = vec![];
        while finality_slot == finality_slot_on_eth || slot <= finality_slot_on_eth {
            if let Ok(block) = relay.get_execution_block_by_slot(slot) {
                blocks.push(block)
            }
            slot += 1;

            finality_slot_on_eth = relay
                .beacon_rpc_client
                .get_last_finalized_slot_number()
                .unwrap()
                .as_u64();
        }

        relay
            .eth_client_contract
            .send_headers(&blocks, finality_slot_on_eth)
            .unwrap();

        finality_slot_on_eth
    }

    fn get_finalized_slot(relay: &Eth2NearRelay) -> u64 {
        relay
            .eth_client_contract
            .get_finalized_beacon_block_slot()
            .unwrap()
    }

    #[test]
    fn test_submit_zero_headers() {
        let mut relay = get_relay(true, true);
        let mut end_slot = get_finalized_slot(&relay);
        end_slot += 1;

        let blocks: Vec<BlockHeader> = vec![];
        if let Ok(_) = relay.eth_client_contract.send_headers(&blocks, end_slot) {
            panic!("No error on submit 0 headers");
        }
    }

    #[test]
    fn test_send_specific_light_client_update() {
        let mut relay = get_relay(true, true);
        let finalized_slot = get_finalized_slot(&relay);

        send_execution_blocks_between(&mut relay, finalized_slot + 1, FINALIZED_SLOT_1);

        let finalized_slot = get_finalized_slot(&relay);
        assert_eq!(finalized_slot, FIRST_SLOT);

        const PATH_TO_LIGHT_CLIENT_UPDATES: &str =
            "../contract_wrapper/data/light_client_updates_kiln_1099394-1099937.json";
        let light_client_updates: Vec<LightClientUpdate> = serde_json::from_str(
            &std::fs::read_to_string(PATH_TO_LIGHT_CLIENT_UPDATES).expect("Unable to read file"),
        )
        .unwrap();
        relay.send_specific_light_cleint_update(light_client_updates[1].clone());

        let finalized_slot = get_finalized_slot(&relay);
        assert_eq!(finalized_slot, FINALIZED_SLOT_1);
    }

    #[test]
    #[ignore]
    fn test_hand_made_light_client_update() {
        let mut relay = get_relay(true, true);
        let finalized_slot = get_finalized_slot(&relay);

        send_execution_blocks_between(&mut relay, finalized_slot + 1, FINALIZED_SLOT_1);

        let finalized_slot = get_finalized_slot(&relay);
        assert_eq!(finalized_slot, FIRST_SLOT);

        relay.send_hand_made_light_client_update(finalized_slot);

        let finalized_slot = get_finalized_slot(&relay);
        assert_eq!(finalized_slot, FINALIZED_SLOT_1);
    }

    #[test]
    #[ignore]
    fn test_hand_made_light_client_update_with_null_signature_slot() {
        let mut relay = get_relay(true, true);
        let finalized_slot = get_finalized_slot(&relay);

        send_execution_blocks_between(&mut relay, finalized_slot + 1, FINALIZED_SLOT_1);

        let finalized_slot = get_finalized_slot(&relay);
        assert_eq!(finalized_slot, FIRST_SLOT);
        let attested_slot = relay.get_attested_slot(finalized_slot + 4).unwrap();
        if relay.get_execution_block_by_slot(attested_slot + 1).is_ok() {
            panic!("Signature slot has block {}", attested_slot + 1);
        }

        relay.send_hand_made_light_client_update(finalized_slot + 4);

        let finalized_slot = get_finalized_slot(&relay);
        assert_eq!(finalized_slot, FINALIZED_SLOT_1);
    }

    #[test]
    fn test_send_light_client_update() {
        let mut relay = get_relay(true, false);
        let finality_slot = get_finalized_slot(&relay);

        let finality_slot_on_eth = send_blocks_till_finalized_eth_slot(&mut relay, finality_slot);
        relay.send_light_client_updates(FIRST_SLOT, finality_slot, finality_slot_on_eth);

        let new_finalized_slot = get_finalized_slot(&relay);
        assert_ne!(finality_slot, new_finalized_slot);
    }

    #[test]
    fn test_get_execution_block_by_slot() {
        let mut relay = get_relay(true, true);
        relay
            .get_execution_block_by_slot(SLOT_WITHOUT_BLOCK - 1)
            .unwrap();
        if let Err(err) = relay.get_execution_block_by_slot(SLOT_WITHOUT_BLOCK) {
            if err.downcast_ref::<NoBlockForSlotError>().is_none() {
                panic!("Wrong error type for slot without block");
            }
        } else {
            panic!("Return execution block for slot without block");
        }

        relay.beacon_rpc_client = BeaconRPCClient::new(
            "http://httpstat.us/504/",
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        if let Err(err) = relay.get_execution_block_by_slot(SLOT_WITHOUT_BLOCK) {
            if err.downcast_ref::<NoBlockForSlotError>().is_some() {
                panic!("Wrong error type for unworking network");
            }
        } else {
            panic!("Return execution block in unworking network");
        }
    }

    #[test]
    fn test_verify_bls_signature() {
        let mut relay = get_relay(true, true);

        const PATH_TO_LIGHT_CLIENT_UPDATES: &str =
            "../contract_wrapper/data/light_client_updates_kiln_1099394-1099937.json";
        let mut light_client_updates: Vec<LightClientUpdate> = serde_json::from_str(
            &std::fs::read_to_string(PATH_TO_LIGHT_CLIENT_UPDATES).expect("Unable to read file"),
        )
        .unwrap();

        assert!(relay
            .verify_bls_signature_for_finality_update(&light_client_updates[1])
            .unwrap());
        light_client_updates[1].attested_beacon_header =
            light_client_updates[0].attested_beacon_header.clone();

        assert!(!relay
            .verify_bls_signature_for_finality_update(&light_client_updates[1])
            .unwrap());
    }

    #[test]
    #[ignore]
    fn test_get_attested_slot() {
        let mut relay = get_relay(true, true);
        let finalized_slot = FINALIZED_SLOT_2;
        let attested_slot = relay.get_attested_slot(finalized_slot).unwrap();

        match HandMadeFinalityLightClientUpdate::get_finality_light_client_update(
            &relay.beacon_rpc_client,
            attested_slot,
            false,
        ) {
            Ok(light_client_update) => {
                let finality_update_slot = light_client_update
                    .finality_update
                    .header_update
                    .beacon_header
                    .slot;

                assert!(finality_update_slot > finalized_slot);
            }
            Err(_) => {
                panic!("Error on get light client update");
            }
        }
    }

    #[test]
    fn test_get_execution_blocks_between() {
        let relay = get_relay(true, true);
        let finalized_slot = get_finalized_slot(&relay);

        let blocks = relay
            .get_execution_blocks_between(finalized_slot + 1, RIGHT_BOUND_IN_SLOT_SEARCH)
            .unwrap();
        assert_eq!(blocks.0.len(), relay.max_submitted_headers as usize);

        let first_block = relay
            .get_execution_block_by_slot(finalized_slot + 1)
            .unwrap();
        assert_eq!(blocks.0[0].hash, first_block.hash);

        for i in 1..blocks.0.len() {
            assert_ne!(blocks.0[i - 1].hash, blocks.0[i].hash);
            assert_eq!(blocks.0[i - 1].hash.unwrap(), blocks.0[i].parent_hash);
        }
    }

    #[test]
    fn test_submit_execution_blocks() {
        let mut relay = get_relay(true, true);
        let mut finalized_slot = get_finalized_slot(&relay);
        let blocks = relay
            .get_execution_blocks_between(finalized_slot + 1, RIGHT_BOUND_IN_SLOT_SEARCH)
            .unwrap();
        relay.submit_execution_blocks(blocks.0, blocks.1, &mut finalized_slot);
        assert_eq!(finalized_slot, blocks.1 - 1);

        let last_slot = relay
            .last_slot_searcher
            .get_last_slot(
                RIGHT_BOUND_IN_SLOT_SEARCH,
                &relay.beacon_rpc_client,
                &relay.eth_client_contract,
            )
            .unwrap();
        assert_eq!(last_slot, blocks.1);

        if relay.get_execution_block_by_slot(last_slot).is_ok() {
            panic!("Wrong last slot");
        }
    }

    #[test]
    #[ignore]
    fn try_submit_update_with_not_enough_blocks() {
        let mut relay = get_relay(true, true);
        let finalized_slot = get_finalized_slot(&relay);

        send_execution_blocks_between(&mut relay, finalized_slot + 1, FINALIZED_SLOT_1 - 1);

        let finalized_slot = get_finalized_slot(&relay);
        assert_eq!(finalized_slot, FIRST_SLOT);

        relay.send_light_client_updates(FIRST_SLOT, finalized_slot, FIRST_SLOT);
        let finalized_slot = get_finalized_slot(&relay);

        assert_eq!(finalized_slot, FIRST_SLOT);
    }

    #[test]
    fn test_not_invalid_attested_slot() {
        let mut relay = get_relay(true, true);
        let finalized_slot = FIRST_SLOT;
        let possible_attested_slot = finalized_slot
            + ONE_EPOCH_IN_SLOTS * 2
            + ONE_EPOCH_IN_SLOTS * relay.light_client_updates_submission_frequency_in_epochs;
        if relay
            .get_execution_block_by_slot(possible_attested_slot)
            .is_ok()
        {
            panic!("possible attested slot has execution block");
        }

        let attested_slot = relay.get_attested_slot(finalized_slot).unwrap();
        relay.get_execution_block_by_slot(attested_slot).unwrap();
    }

    #[test]
    #[should_panic(expected = "504 Gateway Timeout")]
    fn get_execution_blocks_in_bad_network() {
        let mut relay = get_relay(true, true);
        let finalized_slot = get_finalized_slot(&relay);

        relay.beacon_rpc_client = BeaconRPCClient::new(
            "http://httpstat.us/504/",
            TIMEOUT_SECONDS,
            TIMEOUT_STATE_SECONDS,
        );
        relay
            .get_execution_blocks_between(finalized_slot + 1, RIGHT_BOUND_IN_SLOT_SEARCH)
            .unwrap();
    }

    #[test]
    fn test_send_regular_light_client_update() {
        let mut relay = get_relay(true, false);
        let finality_slot = get_finalized_slot(&relay);
        let finality_slot_on_eth = send_blocks_till_finalized_eth_slot(&mut relay, finality_slot);
        relay.send_regular_light_client_update(finality_slot_on_eth, finality_slot);

        let new_finalized_slot = get_finalized_slot(&relay);
        assert_ne!(finality_slot, new_finalized_slot);
    }

    #[test]
    fn test_wrong_last_submitted_slot() {
        let mut relay = get_relay(true, false);
        let finality_slot = get_finalized_slot(&relay);

        let finality_slot_on_eth = send_blocks_till_finalized_eth_slot(&mut relay, finality_slot);
        relay.send_light_client_updates(finality_slot, finality_slot, finality_slot_on_eth + 1);

        let new_finalized_slot = get_finalized_slot(&relay);
        assert_eq!(finality_slot, new_finalized_slot);
    }

    #[test]
    fn test_too_often_updates() {
        let mut relay = get_relay(true, false);
        relay.light_client_updates_submission_frequency_in_epochs = 2;

        let finality_slot = get_finalized_slot(&relay);

        let finality_slot_on_eth = send_blocks_till_finalized_eth_slot(&mut relay, finality_slot);
        relay.send_light_client_updates(finality_slot, finality_slot, finality_slot_on_eth);

        let new_finalized_slot = get_finalized_slot(&relay);
        assert_eq!(finality_slot, new_finalized_slot);
    }

    #[test]
    #[ignore]
    fn test_run() {
        let mut relay = get_relay(true, true);
        let finality_slot = get_finalized_slot(&relay);

        relay.run(Some(5));

        let new_finality_slot = get_finalized_slot(&relay);

        assert_ne!(finality_slot, new_finality_slot);
    }

    #[test]
    #[ignore]
    fn test_base_update_for_new_period() {
        let mut relay = get_relay_from_slot(true, FINALIZED_SLOT_BEFORE_NEW_PERIOD);
        relay.max_submitted_headers = 33;

        let blocks = relay
            .get_execution_blocks_between(
                FINALIZED_SLOT_BEFORE_NEW_PERIOD + 1,
                FINALIZED_SLOT_BEFORE_NEW_PERIOD + 100,
            )
            .unwrap();
        let mut last_slot_on_near = FINALIZED_SLOT_BEFORE_NEW_PERIOD;
        let finality_slot = get_finalized_slot(&relay);

        assert_eq!(finality_slot, last_slot_on_near);

        relay.submit_execution_blocks(blocks.0, blocks.1, &mut last_slot_on_near);

        relay.send_light_client_updates(blocks.1 - 1, last_slot_on_near, blocks.1);

        let new_finality_slot = get_finalized_slot(&relay);

        assert_ne!(FINALIZED_SLOT_BEFORE_NEW_PERIOD, new_finality_slot);
        assert_eq!(BeaconRPCClient::get_period_for_slot(new_finality_slot), 135);
    }

    #[test]
    #[ignore]
    fn test_update_new_period_without_next_sync_committee() {
        let mut relay = get_relay_from_slot(true, FINALIZED_SLOT_BEFORE_NEW_PERIOD);
        relay.max_submitted_headers = 33;
        let blocks = relay
            .get_execution_blocks_between(
                FINALIZED_SLOT_BEFORE_NEW_PERIOD + 1,
                FINALIZED_SLOT_BEFORE_NEW_PERIOD + 100,
            )
            .unwrap();
        let mut last_slot_on_near = FINALIZED_SLOT_BEFORE_NEW_PERIOD;

        relay.submit_execution_blocks(blocks.0, blocks.1, &mut last_slot_on_near);

        let attested_slot = relay
            .get_attested_slot(FINALIZED_SLOT_BEFORE_NEW_PERIOD)
            .unwrap();
        let light_client_update =
            HandMadeFinalityLightClientUpdate::get_finality_light_client_update(
                &relay.beacon_rpc_client,
                attested_slot,
                false,
            )
            .unwrap();

        relay.send_specific_light_cleint_update(light_client_update);

        let new_finality_slot = get_finalized_slot(&relay);

        assert_eq!(FINALIZED_SLOT_BEFORE_NEW_PERIOD, new_finality_slot);
    }

    #[test]
    fn test_send_light_client_update_from_file() {
        let mut relay = get_relay_with_update_from_file(true, true, false);
        let finality_slot = get_finalized_slot(&relay);
        relay.run(None);

        let new_finality_slot = get_finalized_slot(&relay);
        assert_ne!(finality_slot, new_finality_slot);
    }

    #[test]
    fn test_send_light_client_update_from_file_with_next_sync_committee() {
        let mut relay = get_relay_with_update_from_file(true, true, true);
        let finality_slot = get_finalized_slot(&relay);
        relay.run(None);

        let new_finality_slot = get_finalized_slot(&relay);
        assert_ne!(finality_slot, new_finality_slot);
    }

    #[test]
    #[ignore]
    // Can finalize 341 blocks
    fn test_max_finalized_blocks_341() {
        let mut relay = get_relay(true, true);
        relay.max_blocks_for_finalization = 10000;
        relay.max_submitted_headers = 10000;

        let finalized_slot = get_finalized_slot(&relay);
        send_execution_blocks_between(&mut relay, finalized_slot + 1, FINALIZED_SLOT_7);

        const PATH_TO_LIGHT_CLIENT_UPDATES: &str =
            "../contract_wrapper/data/light_client_updates_kiln_1099394-1099937.json";
        let light_client_updates: Vec<LightClientUpdate> = serde_json::from_str(
            &std::fs::read_to_string(PATH_TO_LIGHT_CLIENT_UPDATES).expect("Unable to read file"),
        )
        .unwrap();
        relay.send_specific_light_cleint_update(light_client_updates[7].clone());

        let finalized_slot = get_finalized_slot(&relay);
        assert_eq!(finalized_slot, FINALIZED_SLOT_7);
    }

    #[test]
    #[ignore]
    #[should_panic]
    // Can't finalize 393 blocks
    fn test_max_finalized_blocks_393() {
        let mut relay = get_relay(true, true);
        relay.max_blocks_for_finalization = 10000;
        relay.max_submitted_headers = 10000;

        let finalized_slot = get_finalized_slot(&relay);
        send_execution_blocks_between(&mut relay, finalized_slot + 1, FINALIZED_SLOT_8);

        const PATH_TO_LIGHT_CLIENT_UPDATES: &str =
            "../contract_wrapper/data/light_client_updates_kiln_1099394-1099937.json";
        let light_client_updates: Vec<LightClientUpdate> = serde_json::from_str(
            &std::fs::read_to_string(PATH_TO_LIGHT_CLIENT_UPDATES).expect("Unable to read file"),
        )
        .unwrap();
        relay.send_specific_light_cleint_update(light_client_updates[8].clone());

        let finalized_slot = get_finalized_slot(&relay);
        assert_eq!(finalized_slot, FINALIZED_SLOT_8);
    }
}
