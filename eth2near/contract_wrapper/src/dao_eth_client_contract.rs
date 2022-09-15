use crate::dao_contract::DAOContract;
use crate::dao_types;
use crate::eth_client_contract::EthClientContract;
use crate::eth_client_contract_trait::EthClientContractTrait;
use eth_types::eth2::{LightClientState, LightClientUpdate};
use eth_types::{BlockHeader, H256};
use near_primitives::views::FinalExecutionOutcomeView;
use near_sdk::Balance;
use std::error::Error;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use std::vec::Vec;

pub struct DaoEthClientContract {
    eth_client_contract: EthClientContract,
    dao_contract: DAOContract,
}

impl DaoEthClientContract {
    pub fn new(eth_client_contract: EthClientContract, dao_contract: DAOContract) -> Self {
        Self {
            eth_client_contract,
            dao_contract,
        }
    }
}

impl EthClientContractTrait for DaoEthClientContract {
    fn get_last_submitted_slot(&self) -> u64 {
        self.eth_client_contract.get_last_submitted_slot()
    }

    fn is_known_block(&self, execution_block_hash: &H256) -> Result<bool, Box<dyn Error>> {
        self.eth_client_contract
            .is_known_block(execution_block_hash)
    }

    fn send_light_client_update(
        &mut self,
        light_client_update: LightClientUpdate,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn Error>> {
        // Check for already submitted updates
        let last_proposal_id = self.dao_contract.get_last_proposal_id()?;
        let last_proposal_output = self.dao_contract.get_proposal(last_proposal_id)?;
        if last_proposal_output.proposal.status == dao_types::ProposalStatus::InProgress
            && last_proposal_output.proposal.proposer.to_string()
                == self
                    .dao_contract
                    .contract_wrapper
                    .get_signer_account_id()
                    .to_string()
        {
            return Err(format!(
                "A proposal {} has already been submitted by this relayer which is in progress",
                last_proposal_id
            )
            .into());
        }

        // Submmit new proposal
        let (proposal_id, execution_outcome) =
            self.dao_contract.submit_light_client_update_proposal(
                near_sdk::AccountId::from_str(
                    &self.eth_client_contract.get_account_id().to_string(),
                )?,
                light_client_update,
            )?;

        loop {
            let proposal_status = self.dao_contract.get_proposal(proposal_id);
            if let Ok(staus) = proposal_status {
                if staus.proposal.status != dao_types::ProposalStatus::InProgress {
                    break;
                }
            }

            thread::sleep(Duration::from_secs(10));
        }

        Ok(execution_outcome)
    }

    fn get_finalized_beacon_block_hash(&self) -> Result<H256, Box<dyn Error>> {
        self.eth_client_contract.get_finalized_beacon_block_hash()
    }

    fn get_finalized_beacon_block_slot(&self) -> Result<u64, Box<dyn Error>> {
        self.eth_client_contract.get_finalized_beacon_block_slot()
    }

    fn send_headers(
        &mut self,
        headers: &Vec<BlockHeader>,
        end_slot: u64,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn std::error::Error>> {
        self.eth_client_contract.send_headers(headers, end_slot)
    }

    fn get_min_deposit(&self) -> Result<Balance, Box<dyn Error>> {
        self.eth_client_contract.get_min_deposit()
    }

    fn register_submitter(&self) -> Result<FinalExecutionOutcomeView, Box<dyn Error>> {
        self.eth_client_contract.register_submitter()
    }

    fn get_light_client_state(&self) -> Result<LightClientState, Box<dyn Error>> {
        self.eth_client_contract.get_light_client_state()
    }
}

#[cfg(test)]
mod tests {
    use crate::eth_client_contract_trait::EthClientContractTrait;
    use crate::near_contract_wrapper::NearContractWrapper;
    use crate::{
        dao_contract, dao_eth_client_contract, eth_client_contract, near_contract_wrapper, utils,
    };
    use eth_types::eth2::{ExtendedBeaconBlockHeader, LightClientUpdate, SyncCommittee};
    use eth_types::BlockHeader;
    use std::path::PathBuf;
    use tokio::runtime::Runtime;
    use workspaces::network::DevAccountDeployer;

    fn get_path(path: &str) -> PathBuf {
        let mut json_file_path = std::env::current_exe().unwrap();
        json_file_path.pop();
        json_file_path.push("../../../");
        json_file_path.push(path);

        json_file_path
    }

    #[ignore]
    #[test]
    fn test_smoke_dao_eth_client_contract_wrapper() {
        const PATH_TO_EXECUTION_BLOCKS: &str =
            "data/execution_block_headers_kiln_1099394-1099937.json";
        const PATH_TO_LIGHT_CLIENT_UPDATES: &str =
            "data/light_client_updates_kiln_1099394-1099937.json";
        const PATH_TO_CURRENT_SYNC_COMMITTEE: &str =
            "data/next_sync_committee_kiln_period_133.json";
        const PATH_TO_NEXT_LIGHT_CLIENT_UPDATE: &str =
            "data/next_sync_committee_kiln_period_134.json";

        let execution_blocks_json_file_path = get_path(PATH_TO_EXECUTION_BLOCKS);
        let light_client_update_json_file_path = get_path(PATH_TO_LIGHT_CLIENT_UPDATES);
        let current_sync_committee_path = get_path(PATH_TO_CURRENT_SYNC_COMMITTEE);
        let next_sync_committee_path = get_path(PATH_TO_NEXT_LIGHT_CLIENT_UPDATE);

        let rt = Runtime::new().unwrap();

        let worker = rt.block_on(workspaces::testnet()).unwrap();
        let signer = rt.block_on(worker.dev_create_account()).unwrap();
        let signer_private_key: String =
            utils::trim_quotes(serde_json::to_string(&signer.secret_key()).unwrap());
        let signer_account_id: String = format!("{}", signer.id());

        const NEAR_ENDPOINT: &str = "https://rpc.testnet.near.org";
        const CONTRACT_ACCOUNT_ID: &str = "dev-1660212590113-35162107482173";
        const DAO_CONTRACT_ACCOUNT_ID: &str = "eth2-test.sputnikv2.testnet";

        const NETWORK: &str = "kiln";

        let near_contract_wrapper = Box::new(NearContractWrapper::new_with_raw_secret_key(
            NEAR_ENDPOINT,
            &signer_account_id,
            &signer_private_key,
            CONTRACT_ACCOUNT_ID,
        ));

        let eth_client = eth_client_contract::EthClientContract::new(near_contract_wrapper);

        let execution_blocks: Vec<BlockHeader> = serde_json::from_str(
            &std::fs::read_to_string(execution_blocks_json_file_path).expect("Unable to read file"),
        )
        .unwrap();

        let light_client_updates: Vec<LightClientUpdate> = serde_json::from_str(
            &std::fs::read_to_string(light_client_update_json_file_path)
                .expect("Unable to read file"),
        )
        .unwrap();

        let current_sync_committee: SyncCommittee = serde_json::from_str(
            &std::fs::read_to_string(current_sync_committee_path).expect("Unable to read file"),
        )
        .unwrap();
        let next_sync_committee: SyncCommittee = serde_json::from_str(
            &std::fs::read_to_string(next_sync_committee_path).expect("Unable to read file"),
        )
        .unwrap();

        let finalized_beacon_header = ExtendedBeaconBlockHeader::from(
            light_client_updates[0]
                .clone()
                .finality_update
                .header_update,
        );

        let finalized_hash = light_client_updates[0]
            .clone()
            .finality_update
            .header_update
            .execution_block_hash;
        let mut finalized_execution_header = None::<BlockHeader>;
        for header in &execution_blocks {
            if header.hash.unwrap() == finalized_hash {
                finalized_execution_header = Some(header.clone());
                break;
            }
        }

        eth_client.init_contract(
            NETWORK.to_string(),
            finalized_execution_header.unwrap(),
            finalized_beacon_header,
            current_sync_committee,
            next_sync_committee,
        );

        let dao_contract_wrapper =
            near_contract_wrapper::NearContractWrapper::new_with_raw_secret_key(
                NEAR_ENDPOINT,
                &signer_account_id,
                &signer_private_key,
                DAO_CONTRACT_ACCOUNT_ID,
            );
        let dao_contract = dao_contract::DAOContract::new(Box::new(dao_contract_wrapper));
        let mut dao_client =
            dao_eth_client_contract::DaoEthClientContract::new(eth_client, dao_contract);

        let finalized_slot = dao_client.get_finalized_beacon_block_slot().unwrap();
        let mut next_light_client_update = None;
        for i in 0..light_client_updates.len() {
            if light_client_updates[i]
                .finality_update
                .header_update
                .beacon_header
                .slot
                == finalized_slot
            {
                next_light_client_update = Some(light_client_updates[i + 1].clone());
            }
        }

        for block in &execution_blocks {
            if !dao_client.is_known_block(&block.hash.unwrap()).unwrap() {
                dao_client.send_headers(&vec![block.clone()], 0).unwrap();
            }

            if block.hash.unwrap()
                == next_light_client_update
                    .clone()
                    .unwrap()
                    .finality_update
                    .header_update
                    .execution_block_hash
            {
                dao_client
                    .send_light_client_update(next_light_client_update.unwrap())
                    .unwrap();
                break;
            }
        }
    }
}
