use crate::execution_block_proof::ExecutionBlockProof;
use types::beacon_block_header::BeaconBlockHeader;

pub struct BeaconBlockHeaderWithExecutionData {
    pub beacon_block_header: BeaconBlockHeader,
    pub eth1data_proof: ExecutionBlockProof,
}
