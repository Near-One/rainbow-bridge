use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigForTests {
    pub path_to_current_sync_committee: String,
    pub path_to_next_sync_committee: String,
    pub path_to_light_client_updates: String,
    pub network_name: String,
}

impl ConfigForTests {
    pub fn load_from_toml(path: PathBuf) -> Self {
        let mut config = std::fs::File::open(path).unwrap();
        let mut content = String::new();
        config.read_to_string(&mut content).unwrap();
        toml::from_str(content.as_str()).unwrap()
    }
}
