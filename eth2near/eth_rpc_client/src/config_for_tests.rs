use dotenv::dotenv;
use serde::Deserialize;
use std::env;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigForTests {
    pub beacon_endpoint: String,
    pub eth1_endpoint: String,
    pub first_slot: u64,
    pub eth1_number: u64,
    pub path_to_attested_state_for_period: String,
    pub path_to_light_client_update_for_attested_slot: String,
    pub path_to_block: String,
    pub path_to_header: String,
    pub path_to_light_client_update: String,
}

impl ConfigForTests {
    pub fn load_from_toml(path: PathBuf) -> Self {
        let mut config = std::fs::File::open(path).unwrap();
        let mut content = String::new();
        config.read_to_string(&mut content).unwrap();

        let mut config: Self = toml::from_str(content.as_str()).unwrap();
        dotenv().ok();

        let api_key_string = env::var("ETH1_INFURA_API_KEY").unwrap();
        config.eth1_endpoint = config.eth1_endpoint.replace("API_KEY", &api_key_string);

        config
    }
}
