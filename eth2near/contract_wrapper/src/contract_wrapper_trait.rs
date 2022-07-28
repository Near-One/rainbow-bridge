use near_sdk::Balance;

pub trait ContractWrapper {
    fn call_view_function(&self, method_name: String, args: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn call_change_method(&self, method_name: Vec<String>, args: Vec<Vec<u8>>, deposit: Vec<Balance>) -> Result<(), Box<dyn std::error::Error>>;
}