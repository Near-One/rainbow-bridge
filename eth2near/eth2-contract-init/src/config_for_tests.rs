use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;
use contract_wrapper::eth_network_enum::EthNetwork;

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigForTests {
    pub beacon_endpoint: String,
    pub eth1_endpoint: String,
    pub network_name: EthNetwork,
    pub wasm_filepath: String,
}

impl ConfigForTests {
    pub fn load_from_toml(path: PathBuf) -> Self {
        let mut config = std::fs::File::open(path).unwrap();
        let mut content = String::new();
        config.read_to_string(&mut content).unwrap();
        toml::from_str(content.as_str()).unwrap()
    }
}
