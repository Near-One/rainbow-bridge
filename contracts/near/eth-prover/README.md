# Eth-Prover

Eth-Prover trustless contract to prove the integrity of data (proofs) provided by users and relayer trustlessly.

## How to generate proofs for fastBridge `unlock` method :-

- Pre-requisites before calling `eth_getProof` :
  - DATA, 20 Bytes - address of the account.
  - ARRAY, 32 Bytes - array of storage-keys which should be proofed and  included. See eth_getStorageAt
  - QUANTITY|TAG - integer block number, or the string "latest" or "earliest"

 
- To generate proofs one need to call RPC method `eth_getProof` [check-here](https://eips.ethereum.org/EIPS/eip-1186).
  


## About Parameters of `verify_storage_proof` method :-

- `header_data: Vec<u8>` : Rlp-Serilized Header data from RPC call to `eth_getBlockByNumber`  [check here](https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_getblockbynumber)
- `account_proof: Vec<Vec<u8>>`: Buffer data of account-proof from `eth_getProof` method call response.
- `contract_address: Vec<u8>`: Buffered data of Eth-contract address for which we are prooving.
- `expected_account_state: Vec<u8>`: encoded account state made-up of `{nonce, balance, storageHash, codeHash}`
- `storage_key_hash: Vec<u8>`: keccak256 of storage-key in `eth_getProof`
- `storage_proof: Vec<Vec<u8>>`: Buffer data of `storage-proof` for above `storage_key` from `eth_getProof` method call response.
- `expected_storage_value: Vec<u8>`: storage_value against which proof is to be verified.
- `min_header_height: Option<u64>`: Valid-till block height for unlock.
- `max_header_height: Option<u64>`: Currently set to None
- `skip_bridge_call: bool`: whether to make eth-client call or not. Always take false.