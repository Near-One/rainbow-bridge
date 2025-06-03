use sensitive_url::SensitiveError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LightClientError {
    #[error("Beacon API error: {0:?}")]
    BeaconApi(eth2::Error),

    #[error("URL parse error: {0:?}")]
    UrlParse(SensitiveError),

    #[error("Transform error: {0}")]
    Transform(String),

    #[error("No data found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] borsh::io::Error),

    #[error("Unsupported fork: {0}")]
    UnsupportedFork(String),

    #[error("Beacon block body error: {0}")]
    BeaconBlockBodyError(String),
}

impl From<eth2::Error> for LightClientError {
    fn from(err: eth2::Error) -> Self {
        LightClientError::BeaconApi(err)
    }
}

impl From<SensitiveError> for LightClientError {
    fn from(err: SensitiveError) -> Self {
        LightClientError::UrlParse(err)
    }
}

pub type Result<T> = std::result::Result<T, LightClientError>;
