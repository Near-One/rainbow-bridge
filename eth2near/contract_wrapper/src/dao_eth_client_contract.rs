use crate::dao_contract::DAOContract;
use crate::dao_types;
use crate::eth_client_contract::EthClientContract;
use crate::eth_client_contract_trait::EthClientContractTrait;
use eth_types::eth2::{LightClientState, LightClientUpdate};
use eth_types::{BlockHeader, H256};
use near_primitives::hash::CryptoHash;
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
    ) -> Result<CryptoHash, Box<dyn Error>> {
        let dao_trx_id = self.dao_contract.submit_light_client_update_proposal(
            near_sdk::AccountId::from_str(&self.eth_client_contract.get_account_id().to_string())?,
            light_client_update,
        )?;
        loop {
            let proposal_status = self.dao_contract.get_proposal(dao_trx_id)?;
            if proposal_status.proposal.status != dao_types::ProposalStatus::InProgress {
                break;
            }
            thread::sleep(Duration::from_secs(10));
        }

        Ok(CryptoHash::default())
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
    ) -> Result<CryptoHash, Box<dyn std::error::Error>> {
        self.eth_client_contract.send_headers(headers, end_slot)
    }

    fn get_min_deposit(&self) -> Result<Balance, Box<dyn Error>> {
        self.eth_client_contract.get_min_deposit()
    }

    fn register_submitter(&self) -> Result<CryptoHash, Box<dyn Error>> {
        self.eth_client_contract.register_submitter()
    }

    fn get_light_client_state(&self) -> Result<LightClientState, Box<dyn Error>> {
        self.eth_client_contract.get_light_client_state()
    }
}
