use crate::eth_client_contract::EthClientContract;
use crate::eth_client_contract_trait::EthClientContractTrait;
use eth_types::eth2::{LightClientState, LightClientUpdate};
use eth_types::{BlockHeader, H256};
use near_primitives::views::FinalExecutionOutcomeView;
use near_sdk::Balance;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::vec::Vec;

pub struct FileEthClientContract {
    eth_client_contract: EthClientContract,
    light_client_updates_file: std::fs::File,
    blocks_headers_file: std::fs::File,
}

impl FileEthClientContract {
    pub fn new(eth_client_contract: EthClientContract, dir_path: String) -> Self {
        std::fs::create_dir_all(&dir_path).unwrap();
        let header_path = Path::new(&dir_path).join("execution_block_headers.json");
        let light_client_updates_path = Path::new(&dir_path).join("light_client_updates.json");
        Self {
            eth_client_contract,
            light_client_updates_file: File::create(light_client_updates_path).unwrap(),
            blocks_headers_file: File::create(header_path).unwrap(),
        }
    }
}

impl EthClientContractTrait for FileEthClientContract {
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
        self.light_client_updates_file.write_all(
            serde_json::to_string(&light_client_update)
                .unwrap()
                .as_bytes(),
        )?;
        self.light_client_updates_file.write_all(",".as_bytes())?;
        self.light_client_updates_file.flush()?;

        self.eth_client_contract
            .send_light_client_update(light_client_update)
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
        for header in headers {
            self.blocks_headers_file
                .write_all(serde_json::to_string(&header).unwrap().as_bytes())?;
            self.blocks_headers_file.write_all(",".as_bytes())?;
        }
        self.blocks_headers_file.flush()?;

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
