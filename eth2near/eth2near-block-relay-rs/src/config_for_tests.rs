use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigForTests {
    pub beacon_endpoint: String,
    pub eth1_endpoint: String,
    pub path_to_current_sync_committee: String,
    pub path_to_next_sync_committee: String,
    pub path_to_execution_blocks_headers: String,
    pub path_to_light_client_updates: String,
    pub path_to_attested_state: String,
    pub path_to_finality_state: String,
    pub network_name: String,
    pub first_slot: u64,
    pub slot_without_block: u64,
}

impl ConfigForTests {
    pub fn load_from_toml(path: PathBuf) -> Self {
        let mut config = std::fs::File::open(path).unwrap();
        let mut content = String::new();
        config.read_to_string(&mut content).unwrap();
        toml::from_str(content.as_str()).unwrap()
    }
}