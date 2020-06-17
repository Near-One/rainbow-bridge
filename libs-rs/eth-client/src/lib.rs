use borsh::{BorshDeserialize, BorshSerialize};
use eth_types::*;
use near_bindgen::collections::{Map, Set};
use near_bindgen::near_bindgen;

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

#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct HeaderInfo {
    pub total_difficulty: U256,
    pub parent_hash: H256,
    pub number: u64,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct EthBridge {
    validate_ethash: bool,
    dags_start_epoch: u64,
    dags_merkle_roots: Vec<H128>,

    best_header_hash: H256,
    canonical_header_hashes: Map<u64, H256>,

    headers: Map<H256, BlockHeader>,
    infos: Map<H256, HeaderInfo>,

    recent_header_hashes: Map<u64, Set<H256>>,
}

const NUMBER_OF_BLOCKS_FINALITY: u64 = 30;
const NUMBER_OF_BLOCKS_SAFE: u64 = 10;

impl Default for EthBridge {
    fn default() -> Self {
        near_bindgen::env::panic(b"EthBridge is not initialized");
    }
}

#[near_bindgen]
impl EthBridge {
    #[init]
    pub fn init(
        #[serializer(borsh)] validate_ethash: bool,
        #[serializer(borsh)] dags_start_epoch: u64,
        #[serializer(borsh)] dags_merkle_roots: Vec<H128>,
    ) -> Self {
        assert!(
            near_bindgen::env::state_read::<EthBridge>().is_none(),
            "Already initialized"
        );
        Self {
            validate_ethash,
            dags_start_epoch,
            dags_merkle_roots,

            best_header_hash: Default::default(),
            canonical_header_hashes: Map::new(b"c".to_vec()),

            headers: Map::new(b"h".to_vec()),
            infos: Map::new(b"i".to_vec()),

            recent_header_hashes: Map::new(b"r".to_vec()),
        }
    }

    #[result_serializer(borsh)]
    pub fn initialized(&self) -> bool {
        !self.dags_merkle_roots.is_empty()
    }

    #[result_serializer(borsh)]
    pub fn last_block_number(&self) -> u64 {
        self.infos
            .get(&self.best_header_hash)
            .unwrap_or_default()
            .number
    }

    #[result_serializer(borsh)]
    pub fn dag_merkle_root(&self, #[serializer(borsh)] epoch: u64) -> H128 {
        self.dags_merkle_roots[(&epoch - self.dags_start_epoch) as usize]
    }

    #[result_serializer(borsh)]
    pub fn block_hash(&self, #[serializer(borsh)] index: u64) -> Option<H256> {
        self.canonical_header_hashes.get(&index)
    }

    #[result_serializer(borsh)]
    pub fn block_hash_safe(&self, #[serializer(borsh)] index: u64) -> Option<H256> {
        let best_info = self.infos.get(&self.best_header_hash).unwrap_or_default();
        if best_info.number < index + NUMBER_OF_BLOCKS_SAFE {
            None
        } else {
            self.block_hash(index)
        }
    }

    #[result_serializer(borsh)]
    pub fn add_block_header(
        &mut self,
        #[serializer(borsh)] block_header: Vec<u8>,
        #[serializer(borsh)] dag_nodes: Vec<DoubleNodeWithMerkleProof>,
    ) {
        let header: BlockHeader = rlp::decode(block_header.as_slice()).unwrap();

        if self.best_header_hash == Default::default() {
            // Submit very first block, can trust relayer
            self.maybe_store_header(header);
            return;
        }

        let header_hash = header.hash.unwrap();
        if self.infos.get(&header_hash).is_some() {
            near_bindgen::env::log(
                format!("The header #{} is already known.", header.number).as_bytes(),
            );
            // The header is already known
            return;
        }

        let prev = self
            .headers
            .get(&header.parent_hash)
            .expect("Parent header should be present to add a new header");

        assert!(
            Self::verify_header(&self, &header, &prev, &dag_nodes),
            "The new header {} should be valid", header.number
        );

        self.maybe_store_header(header);
    }
}

impl EthBridge {
    /// Maybe stores a valid header in the contract.
    fn maybe_store_header(&mut self, header: BlockHeader) {
        let best_info = self.infos.get(&self.best_header_hash).unwrap_or_default();
        if best_info.number > header.number + NUMBER_OF_BLOCKS_FINALITY {
            near_bindgen::env::log(
                format!(
                    "The header #{} is too old. The latest is #{}",
                    header.number, best_info.number
                )
                .as_bytes(),
            );
            // It's too late to add this block header.
            return;
        }
        let header_hash = header.hash.unwrap();
        self.headers.insert(&header_hash, &header);

        let parent_info = self.infos.get(&header.parent_hash).unwrap_or_default();
        // Have to compute new total difficulty
        let info = HeaderInfo {
            total_difficulty: parent_info.total_difficulty + header.difficulty,
            parent_hash: header.parent_hash.clone(),
            number: header.number,
        };
        self.infos.insert(&header_hash, &info);
        self.add_recent_header_hash(info.number, &header_hash);
        if info.total_difficulty > best_info.total_difficulty
            || (info.total_difficulty == best_info.total_difficulty
                && header.difficulty % 2 == U256::default())
        {
            // The new header is the tip of the new canonical chain.
            // We need to update hashes of the canonical chain to match the new header.
            near_bindgen::env::log(format!(
                "The received header #{} is the tip of the new canonical chain. There are total {} header hashes",
                info.number,
                self.canonical_header_hashes.len(),
            ).as_bytes());

            // If the new header has a lower number than the previous header, we need to cleaning
            // it going forward.
            if best_info.number > info.number {
                for number in info.number + 1..=best_info.number {
                    self.canonical_header_hashes.remove(&number);
                }
            }
            // Replacing the global best header hash.
            self.best_header_hash = header_hash;
            self.canonical_header_hashes
                .insert(&info.number, &header_hash);

            // Replacing past hashes until we converge into the same parent.
            // Starting from the parent hash.
            let mut number = header.number - 1;
            let mut current_hash = info.parent_hash;
            loop {
                let prev_value = self.canonical_header_hashes.insert(&number, &current_hash);
                // If the current block hash is 0 (unlikely), or the previous hash matches the
                // current hash, then we chains converged and can stop now.
                if number == 0 || prev_value == Some(current_hash) {
                    break;
                }
                // Check if there is an info to get the parent hash
                if let Some(info) = self.infos.get(&current_hash) {
                    current_hash = info.parent_hash;
                } else {
                    break;
                }
                number -= 1;
            }

            self.maybe_gc(best_info.number, info.number);
        } else {
            near_bindgen::env::log(
                format!(
                    "The received header #{} doesn't have the best total difficulty.",
                    info.number
                )
                .as_bytes(),
            );
        }
    }

    /// Removes old headers beyond the finality.
    fn maybe_gc(&mut self, last_best_number: u64, new_best_number: u64) {
        if new_best_number > last_best_number && last_best_number >= NUMBER_OF_BLOCKS_FINALITY {
            for number in last_best_number - NUMBER_OF_BLOCKS_FINALITY
                ..new_best_number - NUMBER_OF_BLOCKS_FINALITY
            {
                if let Some(mut hashes) = self.recent_header_hashes.get(&number) {
                    near_bindgen::env::log(
                        format!("Removing {} old header(s) at #{}", hashes.len(), number)
                            .as_bytes(),
                    );
                    for hash in hashes.iter() {
                        self.infos.remove(&hash);
                        self.headers.remove(&hash);
                    }
                    hashes.clear();
                    self.recent_header_hashes.remove(&number);
                }
            }
            near_bindgen::env::log(
                format!("There are {} headers remaining", self.headers.len()).as_bytes(),
            );
        }
    }

    fn add_recent_header_hash(&mut self, number: u64, hash: &H256) {
        let mut hashes = self.recent_header_hashes.get(&number).unwrap_or_else(|| {
            let mut set_id = Vec::with_capacity(9);
            set_id.extend_from_slice(b"s");
            set_id.extend(number.to_le_bytes().iter());
            Set::new(set_id)
        });
        hashes.insert(&hash);
        self.recent_header_hashes.insert(&number, &hashes);
    }

    fn verify_header(
        &self,
        header: &BlockHeader,
        prev: &BlockHeader,
        dag_nodes: &[DoubleNodeWithMerkleProof],
    ) -> bool {
        let (_mix_hash, result) = Self::hashimoto_merkle(
            self,
            &header.partial_hash.unwrap(),
            &header.nonce,
            header.number,
            dag_nodes,
        );

        //
        // See YellowPaper formula (50) in section 4.3.4
        // 1. Simplified difficulty check to conform adjusting difficulty bomb
        // 2. Added condition: header.parent_hash() == prev.hash()
        //
        ethereum_types::U256::from((result.0).0) < ethash::cross_boundary(header.difficulty.0)
            && (!self.validate_ethash
                || (header.difficulty < header.difficulty * 101 / 100
                    && header.difficulty > header.difficulty * 99 / 100))
            && header.gas_used <= header.gas_limit
            && header.gas_limit < prev.gas_limit * 1025 / 1024
            && header.gas_limit > prev.gas_limit * 1023 / 1024
            && header.gas_limit >= U256(5000.into())
            && header.timestamp > prev.timestamp
            && header.number == prev.number + 1
            && header.parent_hash == prev.hash.unwrap()
    }

    fn hashimoto_merkle(
        &self,
        header_hash: &H256,
        nonce: &H64,
        block_number: u64,
        nodes: &[DoubleNodeWithMerkleProof],
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
                if idx % 2 == 0 && self.validate_ethash {
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
