use borsh::{BorshDeserialize, BorshSerialize};
use eth_types::*;
use near_bindgen::near_bindgen;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct EthProver {
    bridge_smart_contract: String,
}

#[near_bindgen]
impl EthProver {
    pub fn init(&mut self, bridge_smart_contract: String) {
        assert_eq!(self.bridge_smart_contract.len(), 0);
        self.bridge_smart_contract = bridge_smart_contract;
    }

    pub fn verify_log_entry(
        &self,
        log_entry_data: Vec<u8>,
        receipt_data: Vec<u8>,
        header_data: Vec<u8>,
        receipt_proof: Vec<H256>,
    ) -> bool {
        let log_entry: LogEntry = rlp::decode(log_entry_data.as_slice()).unwrap();
        let receipt: Receipt = rlp::decode(receipt_data.as_slice()).unwrap();
        let header_data: BlockHeader = rlp::decode(header_data.as_slice()).unwrap();

        true
    }
}
