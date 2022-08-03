use near_sdk::Balance;
use near_primitives::hash::CryptoHash;

pub trait ContractWrapper {
    fn call_view_function(&self, method_name: String, args: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn call_change_method(&self, method_name: Vec<String>, args: Vec<Vec<u8>>, deposit: Vec<Balance>) -> Result<CryptoHash, Box<dyn std::error::Error>>;
}