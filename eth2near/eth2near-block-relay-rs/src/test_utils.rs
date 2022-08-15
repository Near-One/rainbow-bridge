use std::{thread, time};
use contract_wrapper::eth_client_contract::EthClientContract;
use eth_types::BlockHeader;
use eth_types::eth2::{ExtendedBeaconBlockHeader, LightClientUpdate, SyncCommittee};

pub fn read_json_file_from_data_dir(file_name: &str) -> std::string::String {
    let mut json_file_path = std::env::current_exe().unwrap();
    json_file_path.pop();
    json_file_path.push("../../../data");
    json_file_path.push(file_name);

    std::fs::read_to_string(json_file_path).expect("Unable to read file")
}


pub fn init_contract_from_files(eth_client_contract: &mut EthClientContract) {
    const PATH_TO_CURRENT_SYNC_COMMITTEE: &str = "../contract_wrapper/data/next_sync_committee_133.json";
    const PATH_TO_NEXT_SYNC_COMMITTEE: &str = "../contract_wrapper/data/next_sync_committee_134.json";
    const NETWORK: &str = "kiln";
    const PATH_TO_EXECUTION_BLOCKS: &str = "../contract_wrapper/data/execution_block_headers_kiln_1099394-1099937.json";
    const PATH_TO_LIGHT_CLIENT_UPDATES: &str = "../contract_wrapper/data/light_client_updates_kiln_1099394-1099937.json";

    let execution_blocks: Vec<BlockHeader> = serde_json::from_str(
        &std::fs::read_to_string(PATH_TO_EXECUTION_BLOCKS).expect("Unable to read file"),
    ).unwrap();

    let light_client_updates: Vec<LightClientUpdate> = serde_json::from_str(
        &std::fs::read_to_string(PATH_TO_LIGHT_CLIENT_UPDATES).expect("Unable to read file"),
    ).unwrap();


    let current_sync_committee: SyncCommittee = serde_json::from_str(&std::fs::read_to_string(PATH_TO_CURRENT_SYNC_COMMITTEE).expect("Unable to read file")).unwrap();
    let next_sync_committee: SyncCommittee = serde_json::from_str(&std::fs::read_to_string(PATH_TO_NEXT_SYNC_COMMITTEE).expect("Unable to read file")).unwrap();

    let finalized_beacon_header = ExtendedBeaconBlockHeader::from(light_client_updates[0].clone().finality_update.header_update);

    let finalized_hash = light_client_updates[0].clone().finality_update.header_update.execution_block_hash;
    let mut finalized_execution_header = None::<BlockHeader>;
    for header in &execution_blocks {
        if header.hash.unwrap() == finalized_hash {
            finalized_execution_header = Some(header.clone());
            break;
        }
    }

    eth_client_contract.init_contract(NETWORK.to_string(), finalized_execution_header.unwrap(), finalized_beacon_header, current_sync_committee, next_sync_committee);
    thread::sleep(time::Duration::from_secs(30));
}