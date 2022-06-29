use crate::eth1data_proof::Eth1DataProof;
use types::beacon_block_header::BeaconBlockHeader;

pub struct BeaconBlockHeaderWithExecutionData {
    pub beacon_block_header: BeaconBlockHeader,
    pub eth1data_proof: Eth1DataProof,
}
