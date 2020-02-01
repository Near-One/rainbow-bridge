using near_bindgen::AccountId;
using eth_types::*;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct EthProver {
    bridge_smart_contract: AccountId;
}

#[near_bindgen]
impl EthProver {
    pub fn init(&mut self, bridge_smart_contract: AccountId) {
        assert_eq!(self.bridge_smart_contract, "");
        self.bridge_smart_contract = bridge_smart_contract;
    }

    pub fn verify_event(
        &self,
        event: Vec<u8>,
        receipt: Vec<u8>,
        header: Vec<u8>,
        receipt_proof: Vec<H256>,
    ) -> bool {
        return true;
    }
}
