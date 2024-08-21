use near_sdk::Balance;

pub struct OmniAddress {
    pub chain: String,
    pub account: String
}

pub struct BridgeMessage {
    pub token_id: OmniAddress,
    pub sender: OmniAddress,
    pub receiver: OmniAddress,
    pub amount: Balance
}
