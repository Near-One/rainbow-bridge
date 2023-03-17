use std::error::Error;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct ExecutionPayloadError;

impl Display for ExecutionPayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Execution Payload not found. \
        Beacon Block Body in this blockchain variant doesn't contain execution payload. \
        Please use The Merge variants"
        )
    }
}

impl Error for ExecutionPayloadError {}

#[derive(Debug)]
pub struct FailOnGettingJson {
    pub response: String,
}

impl Display for FailOnGettingJson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fail on getting JSON, response: {}", self.response)
    }
}

impl Error for FailOnGettingJson {}

#[derive(Debug)]
pub struct MissSyncAggregationError;

impl Display for MissSyncAggregationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sync Aggregation not found. \
        Beacon Block Body in this blockchain variant doesn't contain sync aggregation. \
        Please use Altair or The Merge variants"
        )
    }
}

impl Error for MissSyncAggregationError {}

#[derive(Debug)]
pub struct NoBlockForSlotError;

impl Display for NoBlockForSlotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No block found for slot")
    }
}

impl Error for NoBlockForSlotError {}

#[derive(Debug)]
pub struct SignatureSlotNotFoundError;

impl Display for SignatureSlotNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature slot not found!")
    }
}

impl Error for SignatureSlotNotFoundError {}

#[derive(Debug)]
pub struct ErrorOnUnwrapSignatureBit;

impl Display for ErrorOnUnwrapSignatureBit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error on getting signature bits from sync aggregate!")
    }
}

impl Error for ErrorOnUnwrapSignatureBit {}

#[derive(Debug)]
pub struct MissNextSyncCommittee;

impl Display for MissNextSyncCommittee {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Miss next sync committee in the Beacon Block State")
    }
}

impl Error for MissNextSyncCommittee {}

#[derive(Debug)]
pub struct ErrorOnJsonParse;

impl Display for ErrorOnJsonParse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error during json parse")
    }
}

impl Error for ErrorOnJsonParse {}

#[derive(Debug)]
pub struct MissExecutionPayload;

impl Display for MissExecutionPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Miss execution payload")
    }
}

impl Error for MissExecutionPayload {}

#[derive(Debug)]
pub struct MerkleTreeError(pub merkle_proof::MerkleTreeError);

impl Display for MerkleTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Merkle Tree Error: {:?}", self.0)
    }
}

impl Error for MerkleTreeError {}
