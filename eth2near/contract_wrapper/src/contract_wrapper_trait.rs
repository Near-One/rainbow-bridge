use near_primitives::types::AccountId;
use near_primitives::views::FinalExecutionOutcomeView;
use near_sdk::{Balance, Gas};

/// `ContractWrapper` trait provides API for interaction with Contract on NEAR.
pub trait ContractWrapper {
    /// Returns account ID of the contract
    fn get_account_id(&self) -> AccountId;

    /// Returns the address of the account that signs transactions and sends them to the contract
    fn get_signer_account_id(&self) -> AccountId;

    /// Calls the view function in the contract. Returns serialized result of view function execution in the contract or an error.
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

    /// Calls multiple change methods in a batch. Returns the final execution outcome or an error.
    ///
    /// # Arguments
    ///
    /// * `methods_names` - The list of the names of the change methods to call.
    /// * `args` - The list of the serialized arguments for correspondent methods.
    /// * `deposit` - The list of the correspondent deposit for each of the methods.
    /// * `gas` - The gas limit for each of the methods in the batch.
    fn call_change_method_batch(
        &self,
        methods_names: Vec<String>,
        args: Vec<Vec<u8>>,
        deposit: Option<Vec<Balance>>,
        gas: Option<Gas>,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn std::error::Error>>;

    /// Calls one change method. Returns the final execution outcome or an error.
    ///
    /// # Arguments
    ///
    /// * `method_name` - The name of the change method to call.
    /// * `args` - Serialized arguments for the change method.
    /// * `deposit` - Deposit for the change method call.
    /// * `gas` - The gas limit for the transaction.
    fn call_change_method(
        &self,
        method_name: String,
        args: Vec<u8>,
        deposit: Option<Balance>,
        gas: Option<Gas>,
    ) -> Result<FinalExecutionOutcomeView, Box<dyn std::error::Error>>;
}
