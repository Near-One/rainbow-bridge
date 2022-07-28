use std::error::Error;
use eth_types::eth2::{ExtendedBeaconBlockHeader, LightClientUpdate, SyncCommittee};
use std::vec::Vec;
use std::string::String;
use borsh::BorshDeserialize;
use eth_types::{BlockHeader, H256};
use near_primitives::types::AccountId;
use serde_json::json;
use near_primitives::borsh::BorshSerialize;
use std::option::Option;
use near_sdk::ONE_NEAR;
use contract_wrapper::contract_wrapper_trait::ContractWrapper;
use contract_wrapper::near_contract_wrapper::NearContractWrapper;

pub struct EthClientContract {
    last_slot: u64,
    near_contract_wrapper: NearContractWrapper,
    account_id: String,
}

impl EthClientContract {
    pub fn new(near_endpoint: &str, account_id: &str,
               path_to_signer_secret_key: &str, contract_account_id: &str) -> Self {
        EthClientContract {
            last_slot: 0,
            near_contract_wrapper: NearContractWrapper::new(near_endpoint, account_id,
                                                            path_to_signer_secret_key,
                                                            contract_account_id),
            account_id: account_id.to_string(),
        }
    }

    pub fn get_last_submitted_slot(&self) -> u64 {
        return self.last_slot;
    }

    pub fn is_known_block(&self, execution_block_hash: &H256) -> Result<bool, Box<dyn Error>> {
        let result = self.near_contract_wrapper.call_view_function("is_known_execution_header".to_string(), execution_block_hash.try_to_vec()?)?;
        let is_known: bool = bool::try_from_slice(&result)?;
        Ok(is_known)
    }

    pub fn send_light_client_update(& mut self, light_client_update: LightClientUpdate) -> Result<(), Box<dyn Error>> {
        self.near_contract_wrapper.call_change_method(vec!["submit_beacon_chain_light_client_update".to_string()], vec![light_client_update.try_to_vec()?], vec![0])
    }

    pub fn get_finalized_beacon_block_hash(&self) -> Result<H256, Box<dyn Error>> {
        let result = self.near_contract_wrapper.call_view_function("finalized_beacon_block_root".to_string(), json!({}).to_string().into_bytes())?;
        let beacon_block_hash: H256 = H256::try_from_slice(&result)?;
        Ok(beacon_block_hash)
    }

    pub fn send_headers(& mut self, headers: &Vec<BlockHeader>, end_slot: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.last_slot = end_slot;

        let method_names = vec!["submit_execution_header".to_string(); headers.len()];
        let mut args = Vec::new();
        let deposits = vec![0 as u128; headers.len()];

        for header in headers {
            args.push(header.try_to_vec()?);
        }
        self.near_contract_wrapper.call_change_method(method_names, args, deposits)?;
        Ok(())
    }

    pub fn register(&self) -> Result<(), Box<dyn Error>> {
        self.near_contract_wrapper.call_change_method(vec!["register_submitter".to_string()], vec![json!({
            "account_id": self.account_id,
        }).to_string().into_bytes()], vec![10*ONE_NEAR])
    }

    pub fn init_contract(&self, network: String, finalized_execution_header: BlockHeader,
                         finalized_beacon_header: ExtendedBeaconBlockHeader,
                         current_sync_committee: SyncCommittee,
                         next_sync_committee: SyncCommittee) {
        #[derive(BorshSerialize)]
        pub struct InitInput {
            pub network: String,
            pub finalized_execution_header: BlockHeader,
            pub finalized_beacon_header: ExtendedBeaconBlockHeader,
            pub current_sync_committee: SyncCommittee,
            pub next_sync_committee: SyncCommittee,
            pub validate_updates: bool,
            pub verify_bls_signatures: bool,
            pub hashes_gc_threshold: u64,
            pub max_submitted_blocks_by_account: u32,
            pub trusted_signer: Option<AccountId>,
        }

        let init_input = InitInput {
            network,
            finalized_execution_header,
            finalized_beacon_header,
            current_sync_committee,
            next_sync_committee,
            validate_updates: true,
            verify_bls_signatures: false,
            hashes_gc_threshold: 51000,
            max_submitted_blocks_by_account: 8000,
            trusted_signer: Option::<AccountId>::None,
        };

        self.near_contract_wrapper.call_change_method(vec!["init".to_string()], vec![init_input.try_to_vec().unwrap()], vec![0]).unwrap();
    }
}
