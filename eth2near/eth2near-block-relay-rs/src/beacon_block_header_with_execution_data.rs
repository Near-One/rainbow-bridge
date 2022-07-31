use crate::execution_block_proof::ExecutionBlockProof;
use ethereum_types::H256;
use serde::{Deserialize, Serialize};
use std::error::Error;
use types::beacon_block_body::BeaconBlockBody;
use types::beacon_block_header::BeaconBlockHeader;
use types::MainnetEthSpec;

#[derive(Serialize, Deserialize)]
pub struct BeaconBlockHeaderWithExecutionData {
    pub header: BeaconBlockHeader,
    pub execution_block_hash: H256,
    pub execution_hash_branch: [H256; ExecutionBlockProof::PROOF_SIZE],
}

impl BeaconBlockHeaderWithExecutionData {
    pub fn new(
        beacon_block_header: BeaconBlockHeader,
        beacon_block_body: &BeaconBlockBody<MainnetEthSpec>,
    ) -> Result<Self, Box<dyn Error>> {
        let eth1_proof = ExecutionBlockProof::construct_from_beacon_block_body(beacon_block_body)?;

        Ok(Self {
            header: beacon_block_header,
            execution_block_hash: eth1_proof.get_execution_block_hash(),
            execution_hash_branch: eth1_proof.get_proof(),
        })
    }
}
