use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
pub enum NearNetwork {
    Mainnet,
    Testnet,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncorrectNearNetwork;

impl Display for IncorrectNearNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unknown NEAR network id. Possible networks: 'Mainnet', 'Testnet'"
        )
    }
}

impl Error for IncorrectNearNetwork {}

impl Display for NearNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl NearNetwork {
    pub fn as_str(&self) -> &str {
        match self {
            NearNetwork::Mainnet => "mainnet",
            NearNetwork::Testnet => "testnet",
        }
    }
}

impl FromStr for NearNetwork {
    type Err = IncorrectNearNetwork;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(NearNetwork::Mainnet),
            "testnet" => Ok(NearNetwork::Testnet),
            _ => Err(IncorrectNearNetwork),
        }
    }
}
