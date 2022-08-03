use eth_types::eth2::LightClientUpdate;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::{AccountId, Balance, Gas};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;

use crate::contract_wrapper_trait::ContractWrapper;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalOutput {
    /// Id of the proposal.
    pub id: u64,
    #[serde(flatten)]
    pub proposal: Proposal,
}

/// Status of a proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalStatus {
    InProgress,
    /// If quorum voted yes, this proposal is successfully approved.
    Approved,
    /// If quorum voted no, this proposal is rejected. Bond is returned.
    Rejected,
    /// If quorum voted to remove (e.g. spam), this proposal is rejected and bond is not returned.
    /// Interfaces shouldn't show removed proposals.
    Removed,
    /// Expired after period of time.
    Expired,
    /// If proposal was moved to Hub or somewhere else.
    Moved,
    /// If proposal has failed when finalizing. Allowed to re-finalize again to either expire or approved.
    Failed,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct ActionCall {
    pub method_name: String,
    pub args: Base64VecU8,
    pub deposit: U128,
    pub gas: U64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    /// Add member to given role in the policy. This is short cut to updating the whole policy.
    AddMemberToRole { member_id: AccountId, role: String },
    /// Remove member to given role in the policy. This is short cut to updating the whole policy.
    RemoveMemberFromRole { member_id: AccountId, role: String },
    /// Calls `receiver_id` with list of method names in a single promise.
    /// Allows this contract to execute any arbitrary set of actions in other contracts.
    FunctionCall {
        receiver_id: AccountId,
        actions: Vec<ActionCall>,
    },
    /// Indicates that given bounty is done by given user.
    BountyDone {
        bounty_id: u64,
        receiver_id: AccountId,
    },
    /// Just a signaling vote, with no execution.
    Vote,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Vote {
    Approve = 0x0,
    Reject = 0x1,
    Remove = 0x2,
}

/// Proposal that are sent to this DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    /// Original proposer.
    pub proposer: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Count of votes per role per decision: yes / no / spam.
    pub vote_counts: HashMap<String, [Balance; 3]>,
    /// Map of who voted and how.
    pub votes: HashMap<AccountId, Vote>,
    /// Submission time (for voting period).
    pub submission_time: U64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalInput {
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Action {
    /// Action to add proposal. Used internally.
    AddProposal,
    /// Action to remove given proposal. Used for immediate deletion in special cases.
    RemoveProposal,
    /// Vote to approve given proposal or bounty.
    VoteApprove,
    /// Vote to reject given proposal or bounty.
    VoteReject,
    /// Vote to remove given proposal or bounty (because it's spam).
    VoteRemove,
    /// Finalize proposal, called when it's expired to return the funds
    /// (or in the future can be used for early proposal closure).
    Finalize,
    /// Move a proposal to the hub to shift into another DAO.
    MoveToHub,
}

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

    pub fn add_proposal(&mut self, proposal: ProposalInput) -> Result<u64, Box<dyn Error>> {
        let response = self.contract_wrapper.call_change_method(
            "add_proposal".to_string(),
            serde_json::to_string(&proposal).unwrap().into_bytes(),
            Some(1 * near_sdk::ONE_NEAR),
            None,
        )?;

        Ok(serde_json::from_slice(response.as_slice())?)
    }

    pub fn act_proposal(&self, id: u64, action: Action) -> Result<(), Box<dyn Error>> {
        self.contract_wrapper.call_change_method(
            "act_proposal".to_string(),
            json!({ "id": id, "action": action })
                .to_string()
                .into_bytes(),
            Some(1 * near_sdk::ONE_NEAR),
            None,
        )?;
        Ok(())
    }

    pub fn submit_light_client_update_proposale(
        &mut self,
        receiver_id: AccountId,
        update: LightClientUpdate,
    ) -> Result<u64, Box<dyn Error>> {
        let raw_update = update.try_to_vec().unwrap();
        let update_hash = near_primitives::hash::hash(&raw_update);
        let args = Base64VecU8::from(raw_update);

        let action = ActionCall {
            method_name: "submit_beacon_chain_light_client_update".to_string(),
            args: Base64VecU8::from(args.try_to_vec().unwrap()),
            deposit: (1 * near_sdk::ONE_NEAR).into(),
            gas: (290 * Gas::ONE_TERA.0).into(),
        };

        let proposal_input = ProposalInput {
            description: update_hash.to_string(),
            kind: ProposalKind::FunctionCall {
                receiver_id,
                actions: vec![action],
            },
        };

        self.add_proposal(proposal_input)
    }
}
