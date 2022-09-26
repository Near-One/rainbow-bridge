use near_primitives::types::AccountId;
use near_primitives::views::FinalExecutionOutcomeView;
use near_sdk::{Balance, Gas};

/// Contract Wrapper trait provide API for interaction with Contract on NEAR
pub trait ContractWrapper {
    /// Return the contract account address
    fn get_account_id(&self) -> AccountId;

    /// Return the address of account which signed transactions to the contract
    fn get_signer_account_id(&self) -> AccountId;

    /// Call the view function in the contract. Returns serialized result of view contract function
    ///
    /// # Arguments
    ///
    /// * `method_name` - The name of the view function on contract
    /// * `args` - Serialized arguments for view function
    fn call_view_function(
        &self,
        method_name: String,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>>;

    /// Call a few change methods in batch. Return the transaction status.
    ///
    /// # Arguments
    ///
    /// * `methods_names` - The list of the names of the change methods to call.
    /// * `args` - The list of the serialized arguments for correspondent methods.
    /// * `deposit` - The list of the correspondent deposit for each method.
    /// * `gas` - The gas limit for each transaction.
    fn call_change_method_batch(
        &self,
        methods_names: Vec<String>,
        args: Vec<Vec<u8>>,
        deposit: Option<Vec<Balance>>,
        gas: Option<Gas>,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn std::error::Error>>;

    /// Call one change method. Return the transaction status.
    ///
    /// # Arguments
    ///
    /// * `method_name` - The names of the change method to call.
    /// * `args` - Serialized arguments for change method
    /// * `deposit` - Deposit for the change method call
    /// * `gas` - The gas limit for the transaction
    fn call_change_method(
        &self,
        method_name: String,
        args: Vec<u8>,
        deposit: Option<Balance>,
        gas: Option<Gas>,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn std::error::Error>>;
}
