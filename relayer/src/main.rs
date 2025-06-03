use relayer::beacon::BeaconLightClientService;
use relayer::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to any Lighthouse beacon node
    let service = BeaconLightClientService::new("http://unstable.sepolia.beacon-api.nimbus.team")?;

    // Or connect to a public endpoint
    // let service = BeaconLightClientService::new("https://beacon-nd-123-456-789.p2pify.com")?;

    // Check sync status
    if service.is_syncing().await? {
        println!("Beacon node is still syncing...");
        return Ok(());
    }

    // Get the current period
    let last_finalized_slot = service.get_last_finalized_slot().await?;
    let current_period = BeaconLightClientService::get_period_for_slot(last_finalized_slot);

    println!("Current sync committee period: {}", current_period);

    // // Fetch latest finality update
    // match service.fetch_finality_update().await {
    //     Ok(update) => {
    //         println!("✅ Fetched finality update:");
    //         println!("  Signature slot: {}", update.signature_slot);
    //         println!("  Attested slot: {}", update.attested_beacon_header.slot);

    //         // Serialize to BORSH
    //         let borsh_data = borsh::to_vec(&update)?;
    //         println!("  BORSH size: {} bytes", borsh_data.len());
    //     }
    //     Err(e) => {
    //         eprintln!("❌ Failed to fetch finality update: {}", e);
    //         return Err(e);
    //     }
    // }

    // Fetch period update
    match service.fetch_period_update(current_period).await {
        Ok(update) => {
            println!("✅ Fetched period update for period {}:", current_period);
            println!(
                "  Has sync committee update: {}",
                update.next_sync_committee.is_some()
            );
        }
        Err(e) => {
            eprintln!("❌ Failed to fetch period update: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::{BorshDeserialize, BorshSerialize, from_slice, to_vec};

    #[tokio::test]
    async fn test_lighthouse_client() -> Result<()> {
        // Test against a local Lighthouse node
        let service =
            BeaconLightClientService::new("http://unstable.sepolia.beacon-api.nimbus.team")?;

        // Check if node is syncing
        let is_syncing = service.is_syncing().await?;
        println!("Node is syncing: {}", is_syncing);

        // Get current period
        let last_finalized_slot = service.get_last_finalized_slot().await?;
        let current_period = BeaconLightClientService::get_period_for_slot(last_finalized_slot);

        // Fetch update for current period
        match service.fetch_period_update(928).await {
            Ok(update) => {
                println!("Successfully fetched update for period {}", current_period);
                println!("Attested slot: {}", update.attested_header.beacon.slot);
                println!("{:#?}", update);

                // Serialize to borsh
                //let borsh_data = to_vec(&update).unwrap();
                //println!("BORSH size: {} bytes", borsh_data.len());

                // print hex borsh data
                //println!("BORSH data: {}", hex::encode(borsh_data));
            }
            Err(error) => {
                eprintln!(
                    "Failed to fetch update for period {}: {:?}",
                    current_period, error
                );
                // Or for more detailed error info:
                // eprintln!("Failed to fetch update for period {}: {}", current_period, error);
            }
        }

        Ok(())
    }

    // #[tokio::test]
    // async fn test_finality_update() -> Result<()> {
    //     let service = BeaconLightClientService::new("http://localhost:5052")?;

    //     if let Ok(update) = service.fetch_finality_update().await {
    //         println!("Successfully fetched finality update");
    //         println!("Signature slot: {}", update.signature_slot);
    //     }

    //     Ok(())
    // }
}
