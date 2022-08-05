use crate::dao_types::*;
use eth_types::eth2::LightClientUpdate;
use near_sdk::borsh::BorshSerialize;
use near_sdk::json_types::Base64VecU8;
use near_sdk::{AccountId, Gas};
use serde_json::json;
use std::error::Error;

use crate::contract_wrapper_trait::ContractWrapper;
pub struct DAOContract {
    contract_wrapper: Box<dyn ContractWrapper>,
}

impl DAOContract {
    pub fn new(contract_wrapper: Box<dyn ContractWrapper>) -> Self {
        DAOContract { contract_wrapper }
    }

    pub fn get_last_proposal_id(&self) -> Result<u64, Box<dyn Error>> {
        let response = self
            .contract_wrapper
            .call_view_function("get_last_proposal_id".to_string(), vec![])?;

        Ok(serde_json::from_slice(response.as_slice())?)
    }

    pub fn get_proposal(&self, id: u64) -> Result<ProposalOutput, Box<dyn Error>> {
        let response = self.contract_wrapper.call_view_function(
            "get_proposal".to_string(),
            json!({ "id": id }).to_string().into_bytes(),
        )?;

        Ok(serde_json::from_slice(response.as_slice())?)
    }

    pub fn get_policy(&self) -> Result<Policy, Box<dyn Error>> {
        let response = self
            .contract_wrapper
            .call_view_function("get_policy".to_string(), json!({}).to_string().into_bytes())?;

        Ok(serde_json::from_slice(response.as_slice())?)
    }

    pub fn add_proposal(&mut self, proposal: ProposalInput) -> Result<u64, Box<dyn Error>> {
        let policy = self.get_policy()?;
        let response = self.contract_wrapper.call_change_method(
            "add_proposal".to_string(),
            json!({ "proposal": json!(proposal) })
                .to_string()
                .into_bytes(),
            Some(policy.proposal_bond.0),
            None,
        )?;

        Ok(serde_json::from_slice(
            response
                .status
                .as_success_decoded()
                .ok_or("Failed to add proposal")?
                .as_slice(),
        )?)
    }

    pub fn act_proposal(&self, id: u64, action: Action) -> Result<(), Box<dyn Error>> {
        self.contract_wrapper.call_change_method(
            "act_proposal".to_string(),
            json!({ "id": id, "action": action })
                .to_string()
                .into_bytes(),
            None,
            None,
        )?;
        Ok(())
    }

    pub fn submit_light_client_update_proposal(
        &mut self,
        receiver_id: AccountId,
        update: LightClientUpdate,
    ) -> Result<u64, Box<dyn Error>> {
        let raw_update = update.try_to_vec().unwrap();
        let update_hash = near_primitives::hash::hash(&raw_update);
        let args = Base64VecU8::from(raw_update);

        const GAS_FOR_SUBMIT_LIGHT_CLIENT_UPDATE: u64 = 250 * Gas::ONE_TERA.0;
        let action = ActionCall {
            method_name: "submit_beacon_chain_light_client_update".to_string(),
            args,
            deposit: 0.into(),
            gas: GAS_FOR_SUBMIT_LIGHT_CLIENT_UPDATE.into(),
        };

        let proposal_input = ProposalInput {
            description: json!({
                "finalized slot": update.finality_update.header_update.beacon_header.slot.to_string(), 
                "update_hash": update_hash.to_string() 
            }).to_string(),
            kind: ProposalKind::FunctionCall {
                receiver_id,
                actions: vec![action],
            },
        };

        self.add_proposal(proposal_input)
    }
}
