use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
pub enum EthNetwork {
    Mainnet,
    Kiln,
    Ropsten,
    Goerli,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncorrectEthNetwork;

impl Display for IncorrectEthNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unknown Ethereum network. Possible networks: 'Mainnet', 'Kiln', 'Goerli', 'Ropsten'"
        )
    }
}

impl Error for IncorrectEthNetwork {}

impl Display for EthNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl EthNetwork {
    pub fn as_str(&self) -> &str {
        match self {
            EthNetwork::Mainnet => "mainnet",
            EthNetwork::Kiln => "kiln",
            EthNetwork::Goerli => "goerli",
            EthNetwork::Ropsten => "ropsten",
        }
    }
}

impl FromStr for EthNetwork {
    type Err = IncorrectEthNetwork;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(EthNetwork::Mainnet),
            "kiln" => Ok(EthNetwork::Kiln),
            "goerli" => Ok(EthNetwork::Goerli),
            "ropsten" => Ok(EthNetwork::Ropsten),
            _ => Err(IncorrectEthNetwork),
        }
    }
}
