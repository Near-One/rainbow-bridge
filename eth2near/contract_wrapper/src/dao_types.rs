use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base58CryptoHash, Base64VecU8, U128, U64};
use near_sdk::AccountId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Account ID that represents a token in near-sdk v3.
/// Need to keep it around for backward compatibility.
pub type OldAccountId = String;

/// Information recorded about claim of the bounty by given user.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BountyClaim {
    /// Bounty id that was claimed.
    bounty_id: u64,
    /// Start time of the claim.
    start_time: U64,
    /// Deadline specified by claimer.
    deadline: U64,
    /// Completed?
    completed: bool,
}

/// Bounty information.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Bounty {
    /// Description of the bounty.
    pub description: String,
    /// Token the bounty will be paid out.
    /// Can be "" for $NEAR or a valid account id.
    pub token: OldAccountId,
    /// Amount to be paid out.
    pub amount: U128,
    /// How many times this bounty can be done.
    pub times: u32,
    /// Max deadline from claim that can be spend on this bounty.
    pub max_deadline: U64,
}

/// Info about factory that deployed this contract and if auto-update is allowed.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct FactoryInfo {
    pub factory_id: AccountId,
    pub auto_update: bool,
}

/// Function call arguments.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct PolicyParameters {
    pub proposal_bond: Option<U128>,
    pub proposal_period: Option<U64>,
    pub bounty_bond: Option<U128>,
    pub bounty_forgiveness_period: Option<U64>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
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

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    /// Name of the DAO.
    pub name: String,
    /// Purpose of this DAO.
    pub purpose: String,
    /// Generic metadata. Can be used by specific UI to store additional data.
    /// This is not used by anything in the contract.
    pub metadata: Base64VecU8,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    /// Change the DAO config.
    ChangeConfig { config: Config },
    /// Change the full policy.
    ChangePolicy { policy: VersionedPolicy },
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
    /// Upgrade this contract with given hash from blob store.
    UpgradeSelf { hash: Base58CryptoHash },
    /// Upgrade another contract, by calling method with the code from given hash from blob store.
    UpgradeRemote {
        receiver_id: AccountId,
        method_name: String,
        hash: Base58CryptoHash,
    },
    /// Transfers given amount of `token_id` from this DAO to `receiver_id`.
    /// If `msg` is not None, calls `ft_transfer_call` with given `msg`. Fails if this base token.
    /// For `ft_transfer` and `ft_transfer_call` `memo` is the `description` of the proposal.
    Transfer {
        /// Can be "" for $NEAR or a valid account id.
        token_id: OldAccountId,
        receiver_id: AccountId,
        amount: U128,
        msg: Option<String>,
    },
    /// Sets staking contract. Can only be proposed if staking contract is not set yet.
    SetStakingContract { staking_id: AccountId },
    /// Add new bounty.
    AddBounty { bounty: Bounty },
    /// Indicates that given bounty is done by given user.
    BountyDone {
        bounty_id: u64,
        receiver_id: AccountId,
    },
    /// Just a signaling vote, with no execution.
    Vote,
    /// Change information about factory and auto update.
    FactoryInfoUpdate { factory_info: FactoryInfo },
    /// Add new role to the policy. If the role already exists, update it. This is short cut to updating the whole policy.
    ChangePolicyAddOrUpdateRole { role: RolePermission },
    /// Remove role from the policy. This is short cut to updating the whole policy.
    ChangePolicyRemoveRole { role: String },
    /// Update the default vote policy from the policy. This is short cut to updating the whole policy.
    ChangePolicyUpdateDefaultVotePolicy { vote_policy: VotePolicy },
    /// Update the parameters from the policy. This is short cut to updating the whole policy.
    ChangePolicyUpdateParameters { parameters: PolicyParameters },
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
    pub vote_counts: HashMap<String, [u64; 3]>,
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

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum RoleKind {
    /// Matches everyone, who is not matched by other roles.
    Everyone,
    /// Member greater or equal than given balance. Can use `1` as non-zero balance.
    Member(U128),
    /// Set of accounts.
    Group(HashSet<AccountId>),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct RolePermission {
    /// Name of the role to display to the user.
    pub name: String,
    /// Kind of the role: defines which users this permissions apply.
    pub kind: RoleKind,
    /// Set of actions on which proposals that this role is allowed to execute.
    /// <proposal_kind>:<action>
    pub permissions: HashSet<String>,
    /// For each proposal kind, defines voting policy.
    pub vote_policy: HashMap<String, VotePolicy>,
}

/// How the voting policy votes get weigthed.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum WeightKind {
    /// Using token amounts and total delegated at the moment.
    TokenWeight,
    /// Weight of the group role. Roles that don't have scoped group are not supported.
    RoleWeight,
}

/// Defines configuration of the vote.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct VotePolicy {
    /// Kind of weight to use for votes.
    pub weight_kind: WeightKind,
    /// Minimum number required for vote to finalize.
    /// If weight kind is TokenWeight - this is minimum number of tokens required.
    ///     This allows to avoid situation where the number of staked tokens from total supply is too small.
    /// If RoleWeight - this is minimum number of votes.
    ///     This allows to avoid situation where the role is got too small but policy kept at 1/2, for example.
    pub quorum: U128,
    /// How many votes to pass this vote.
    pub threshold: WeightOrRatio,
}

/// Versioned policy.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde", untagged)]
pub enum VersionedPolicy {
    /// Default policy with given accounts as council.
    Default(Vec<AccountId>),
    Current(Policy),
}

/// Direct weight or ratio to total weight, used for the voting policy.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum WeightOrRatio {
    Weight(U128),
    Ratio(u64, u64),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Policy {
    /// List of roles and permissions for them in the current policy.
    pub roles: Vec<RolePermission>,
    /// Default vote policy. Used when given proposal kind doesn't have special policy.
    pub default_vote_policy: VotePolicy,
    /// Proposal bond.
    pub proposal_bond: U128,
    /// Expiration period for proposals.
    pub proposal_period: U64,
    /// Bond for claiming a bounty.
    pub bounty_bond: U128,
    /// Period in which giving up on bounty is not punished.
    pub bounty_forgiveness_period: U64,
}
