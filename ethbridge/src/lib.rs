use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use std::io::Cursor;
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::{near_bindgen};
use eth_types::*;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;

#[derive(Default, Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct DoubleNodeWithMerkleProof {
    pub dag_nodes: Vec<H512>, // [H512; 2]
    pub proof: Vec<H128>,
}

impl DoubleNodeWithMerkleProof {
    fn truncate_to_h128(arr: H256) -> H128 {
        let mut data = [0u8; 16];
        data.copy_from_slice(&(arr.0).0[16..]);
        H128(data.into())
    }

    fn hash_h128(l: H128, r: H128) -> H128 {
        let mut data = [0u8; 64];
        data[16..32].copy_from_slice(&(l.0).0);
        data[48..64].copy_from_slice(&(r.0).0);
        Self::truncate_to_h128(near_sha256(&data).into())
    }

    pub fn apply_merkle_proof(&self, index: u64) -> H128 {
        let mut data = [0u8; 128];
        data[..64].copy_from_slice(&(self.dag_nodes[0].0).0);
        data[64..].copy_from_slice(&(self.dag_nodes[1].0).0);

        let mut leaf = Self::truncate_to_h128(near_sha256(&data).into());

        for i in 0..self.proof.len() {
            if (index >> i as u64) % 2 == 0 {
                leaf = Self::hash_h128(leaf, self.proof[i]);
            } else {
                leaf = Self::hash_h128(self.proof[i], leaf);
            }
        }
        leaf
    }
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct EthBridge {
    dags_start_epoch: u64,
    dags_merkle_roots: Vec<H128>,
    block_hashes: HashMap<u64, H256>,
    block_difficulties: HashMap<u64, U256>,
    last_block_number: u64,
}

impl EthBridge {
    const NUMBER_OF_FUTURE_BLOCKS: u64 = 10;

    pub fn init(&mut self, dags_start_epoch: u64, dags_merkle_roots: Vec<H128>) {
        assert!(self.dags_merkle_roots.len() == 0 && dags_merkle_roots.len() > 0);
        self.dags_start_epoch = dags_start_epoch;
        self.dags_merkle_roots = dags_merkle_roots;
    }

    pub fn initialized(&self) -> bool {
        self.dags_merkle_roots.len() > 0
    }

    pub fn last_block_number(&self) -> u64 {
        self.last_block_number
    }

    pub fn dag_merkle_root(&self, epoch: u64) -> H128 {
        self.dags_merkle_roots[(&epoch - self.dags_start_epoch) as usize]
    }

    pub fn block_hash_unsafe(&self, index: u64) -> Option<H256> {
        self.block_hashes.get(&index).cloned()
    }

    pub fn block_hash(&self, index: u64) -> Option<H256> {
        if index + EthBridge::NUMBER_OF_FUTURE_BLOCKS > self.last_block_number {
            return Option::None;
        }
        self.block_hashes.get(&index).cloned()
    }

    //
    // Usually each next sequence should start from the lastest added block:
    // [1]-[2]-[3]-[4]
    //             [4]-[5]-[6]-[7]
    //
    // In case of reorg next sequence can start from the lastest common block:
    // [1]-[2]-[3]-[4a]
    //         [3]-[4b]-[5]-[6]
    //
    pub fn add_block_headers(
        &mut self,
        block_headers: Vec<Vec<u8>>,
        dag_nodes: Vec<Vec<DoubleNodeWithMerkleProof>>,
    ) {
        let mut prev: BlockHeader = rlp::decode(block_headers[0].as_slice()).unwrap();

        let very_first_blocks = self.last_block_number == 0;
        if very_first_blocks {
            // Submit very first block, can trust relayer
            self.block_hashes.insert(prev.number, prev.hash.unwrap());
            self.last_block_number = prev.number;
        } else {
            // Check first block hash equals to submitted one
            assert_eq!(prev.hash.unwrap(), self.block_hashes[&prev.number]);
        }

        let mut origin_total_difficulty = U256(0.into());
        let mut branch_total_difficulty = U256(0.into());
        
        // Check validity of all the following blocks
        for i in 1..block_headers.len() {
            let header: BlockHeader = rlp::decode(block_headers[i].as_slice()).unwrap();
            
            assert!(Self::verify_header(
                &self,
                &header,
                &prev,
                &dag_nodes[i]
            ));

            // Compute new chain total difficulty
            branch_total_difficulty += header.difficulty;
            if header.number <= self.last_block_number {
                // Compute old chain total difficulty if reorg
                origin_total_difficulty += self.block_difficulties[&header.number];
            }

            self.block_hashes.insert(header.number, header.hash.unwrap());
            self.block_difficulties.insert(header.number, header.difficulty);
            prev = header;
        }

        if !very_first_blocks {
            // Ensure the longest chain rule: https://ethereum.stackexchange.com/a/13750/3032
            // https://github.com/ethereum/go-ethereum/blob/525116dbff916825463931361f75e75e955c12e2/core/blockchain.go#L863
            assert!(
                branch_total_difficulty > origin_total_difficulty ||
                (
                    branch_total_difficulty == origin_total_difficulty &&
                    prev.difficulty % 2 == U256(0.into()) // hash is good enough random for us
                )
            );
        }
        self.last_block_number = prev.number;
    }

    pub fn verify_header(
        &self,
        header: &BlockHeader,
        prev: &BlockHeader,
        dag_nodes: &Vec<DoubleNodeWithMerkleProof>,
    ) -> bool {
        let (_mix_hash, result) = Self::hashimoto_merkle(
            self,
            header.partial_hash.unwrap(),
            header.nonce,
            header.number,
            dag_nodes.to_vec(),
        );

        //
        // See YellowPaper formula (50) in section 4.3.4
        // 1. Simplified difficulty check to conform adjusting difficulty bomb
        // 2. Added condition: header.parent_hash() == prev.hash()
        //
        ethereum_types::U256::from((result.0).0) < ethash::cross_boundary(header.difficulty.0)
        && header.difficulty < header.difficulty * 101 / 100
        && header.difficulty > header.difficulty * 99 / 100
        && header.gas_used <= header.gas_limit
        && header.gas_limit < prev.gas_limit * 1025 / 1024
        && header.gas_limit > prev.gas_limit * 1023 / 1024
        && header.gas_limit >= U256(5000.into())
        && header.timestamp > prev.timestamp
        && header.number == prev.number + 1
        && header.parent_hash == prev.hash.unwrap()
    }

    pub fn hashimoto_merkle(
        &self,
        header_hash: H256,
        nonce: H64,
        block_number: u64,
        nodes: Vec<DoubleNodeWithMerkleProof>,
    ) -> (H256, H256) {
        // Boxed index since ethash::hashimoto gets Fn, but not FnMut
        let index = std::cell::RefCell::new(0);

        // Reuse single Merkle root across all the proofs
        let merkle_root = self.dag_merkle_root((block_number as usize / 30000) as u64);

        let pair = ethash::hashimoto_with_hasher(
            header_hash.0,
            nonce.0,
            ethash::get_full_size(block_number as usize / 30000),
            |offset| {
                let idx = *index.borrow_mut();
                *index.borrow_mut() += 1;

                // Each two nodes are packed into single 128 bytes with Merkle proof
                let node = &nodes[idx / 2];
                if idx % 2 == 0 {
                    // Divide by 2 to adjust offset for 64-byte words instead of 128-byte
                    assert_eq!(merkle_root, node.apply_merkle_proof((offset / 2) as u64));
                };

                // Reverse each 32 bytes for ETHASH compatibility
                let mut data = (node.dag_nodes[idx % 2].0).0;
                data[..32].reverse();
                data[32..].reverse();
                data.into()
            },
            near_keccak256,
            near_keccak512,
        );

        (H256(pair.0), H256(pair.1))
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn init() {
    near_bindgen::env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    let input = near_bindgen::env::input().unwrap();
    let mut c = Cursor::new(&input);
    let dags_start_epoch: u64 = borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    let dags_merkle_roots: Vec<H128> =
        borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    assert_eq!(c.position(), input.len() as u64, "Not all bytes read from input");
    let mut contract: EthBridge = near_bindgen::env::state_read().unwrap_or_default();
    contract.init(dags_start_epoch, dags_merkle_roots);
    near_bindgen::env::state_write(&contract);
}
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn initialized() {
    near_bindgen::env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    let contract: EthBridge = near_bindgen::env::state_read().unwrap_or_default();
    let result = contract.initialized();
    let result = result.try_to_vec().unwrap();
    near_bindgen::env::value_return(&result);
}
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn last_block_number() {
    near_bindgen::env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    let contract: EthBridge = near_bindgen::env::state_read().unwrap_or_default();
    let result = contract.last_block_number();
    let result = result.try_to_vec().unwrap();
    near_bindgen::env::value_return(&result);
}
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn dag_merkle_root() {
    near_bindgen::env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    let input = near_bindgen::env::input().unwrap();
    let mut c = Cursor::new(&input);
    let epoch: u64 = borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    assert_eq!(c.position(), input.len() as u64, "Not all bytes read from input");
    let contract: EthBridge = near_bindgen::env::state_read().unwrap_or_default();
    let result = contract.dag_merkle_root(epoch);
    let result = result.try_to_vec().unwrap();
    near_bindgen::env::value_return(&result);
}
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn block_hash_unsafe() {
    near_bindgen::env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    let input = near_bindgen::env::input().unwrap();
    let mut c = Cursor::new(&input);
    let index: u64 = borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    assert_eq!(c.position(), input.len() as u64, "Not all bytes read from input");
    let contract: EthBridge = near_bindgen::env::state_read().unwrap_or_default();
    let result = contract.block_hash_unsafe(index);
    let result = result.try_to_vec().unwrap();
    near_bindgen::env::value_return(&result);
}
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn block_hash() {
    near_bindgen::env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    let input = near_bindgen::env::input().unwrap();
    let mut c = Cursor::new(&input);
    let index: u64 = borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    assert_eq!(c.position(), input.len() as u64, "Not all bytes read from input");
    let contract: EthBridge = near_bindgen::env::state_read().unwrap_or_default();
    let result = contract.block_hash(index);
    let result = result.try_to_vec().unwrap();
    near_bindgen::env::value_return(&result);
}
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn add_block_headers() {
    near_bindgen::env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    let input = near_bindgen::env::input().unwrap();
    let mut c = Cursor::new(&input);
    let block_headers: Vec<Vec<u8>> =
        borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    let dag_nodes: Vec<Vec<DoubleNodeWithMerkleProof>> =
        borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    assert_eq!(c.position(), input.len() as u64, "Not all bytes read from input");
    let mut contract: EthBridge = near_bindgen::env::state_read().unwrap_or_default();
    contract.add_block_headers(block_headers, dag_nodes);
    near_bindgen::env::state_write(&contract);
}
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn verify_header() {
    near_bindgen::env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    let input = near_bindgen::env::input().unwrap();
    let mut c = Cursor::new(&input);
    let header: BlockHeader = borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    let prev: BlockHeader = borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    let dag_nodes: Vec<DoubleNodeWithMerkleProof> =
        borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    assert_eq!(c.position(), input.len() as u64, "Not all bytes read from input");
    let contract: EthBridge = near_bindgen::env::state_read().unwrap_or_default();
    let result = contract.verify_header(&header, &prev, &dag_nodes);
    let result = result.try_to_vec().unwrap();
    near_bindgen::env::value_return(&result);
}
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn hashimoto_merkle() {
    near_bindgen::env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
    let input = near_bindgen::env::input().unwrap();
    let mut c = Cursor::new(&input);
    let header_hash: H256 = borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    let nonce: H64 = borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    let block_number: u64 = borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    let nodes: Vec<DoubleNodeWithMerkleProof> =
        borsh::BorshDeserialize::deserialize(&mut c).unwrap();
    assert_eq!(c.position(), input.len() as u64, "Not all bytes read from input");
    let contract: EthBridge = near_bindgen::env::state_read().unwrap_or_default();
    let result = contract.hashimoto_merkle(header_hash, nonce, block_number, nodes);
    let result = result.try_to_vec().unwrap();
    near_bindgen::env::value_return(&result);
}