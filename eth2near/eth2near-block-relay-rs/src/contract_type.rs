use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
pub enum ContractType {
    Near,
    Dao,
    File,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncorrectContractType;

impl Display for IncorrectContractType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unknown contract type. Possible contract types: 'Near', 'Dao', 'File'"
        )
    }
}

impl Error for IncorrectContractType {}

impl Display for ContractType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ContractType {
    pub fn as_str(&self) -> &str {
        match self {
            ContractType::Near => "Near",
            ContractType::Dao => "Dao",
            ContractType::File => "File",
        }
    }
}

impl FromStr for ContractType {
    type Err = IncorrectContractType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "near" => Ok(ContractType::Near),
            "dao" => Ok(ContractType::Dao),
            "file" => Ok(ContractType::File),
            _ => Err(IncorrectContractType),
        }
    }
}
