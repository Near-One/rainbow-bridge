use std::error::Error;
use reqwest::blocking::Client;

pub struct Eth1RPCClient {
    endpoint_url: String,
    client: Client,
}

impl Eth1RPCClient {
    pub fn new(endpoint_url: &str) -> Self {
        Self {
            endpoint_url: endpoint_url.to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn get_block_header_by_number(
        &self,
        number: &str,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn get_block_by_number(&self, number: &str) -> Result<(), Box<dyn Error>>  {
        Ok(())
    }
}

impl Default for Eth1RPCClient {
    fn default() -> Self {
        Self::new("https://rpc.kiln.themerge.dev")
    }
}

#[cfg(test)]
mod tests {
    const TEST_BEACON_BLOCK_ID: u32 = 741888;

    #[test]
    fn test_get_header_from_json() {
    }
}
