use rlp::{Rlp, DecoderError as RlpDecoderError, Decodable as RlpDecodable};
use borsh::{BorshDeserialize, BorshSerialize};
use eth_types::*;
//use near_bindgen::near_bindgen;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
pub enum EthTrieNodeType {
    Empty,
    Branch,
    Leaf,
    Extension,
}

impl Default for EthTrieNodeType {
    fn default() -> EthTrieNodeType {
        EthTrieNodeType::Empty
    }
}

#[derive(Default, Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct EthTrieNode {
    pub raw: Vec<Vec<u8>>,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub node_type: EthTrieNodeType,
}

fn bytesToNibbles(bytes: Vec<u8>) -> Vec<u8> {
    bytes.iter().flat_map(|b| vec![b >> 4, b & 0x0F]).collect()
}

impl RlpDecodable for EthTrieNode {
    fn decode(rlp: &Rlp) -> Result<Self, RlpDecoderError> {
        match rlp.item_count()? {
            17 => Ok(EthTrieNode {
                raw: vec![],
                value: rlp.val_at(16)?,
                key: vec![],
                node_type: EthTrieNodeType::Branch,
            }),
            2 => {
                let list: Vec<u8> = rlp.list_at(0)?;
                Ok(EthTrieNode {
                    raw: vec![],
                    value: rlp.val_at(1)?,
                    key: if (list[0] >> 4) % 2 == 1 {
                        vec![list[0] >> 4]
                    } else {
                        vec![list[0] >> 4, list[0] & 0x0F]
                    },
                    node_type: if (list[0] >> 4) > 1 {
                        EthTrieNodeType::Leaf
                    } else {
                        EthTrieNodeType::Extension
                    },
                })
            }
            0 => Ok(EthTrieNode {
                raw: vec![],
                value: vec![],
                key: vec![],
                node_type: EthTrieNodeType::Empty,
            }),
            _ => {
                panic!("Unreachable code")
            },
        }
    }
}

// impl EthTrieNode {
//     pub fn init(data: Vec<Vec<u8>>) -> Self {
//         if data.len() == 17 {
//             Self {
//                 raw: data.clone(),
//                 value: data[16].clone(),
//                 key: vec![],
//                 node_type: EthTrieNodeType::Branch,
//             }
//         } else
//         if data.len() == 2 {
//             Self {
//                 raw: data.clone(),
//                 value: data[1].clone(),
//                 key: if (data[0][0] >> 4) % 2 == 1 {
//                     vec![data[0][0] >> 4]
//                 } else {
//                     vec![data[0][0] >> 4, data[0][0] & 0x0F]
//                 },
//                 node_type: if (data[0][0] >> 4) > 1 { EthTrieNodeType::Leaf } else { EthTrieNodeType::Extension },
//             }
//         } else
//         if data.len() == 0 {
//             Self {
//                 raw: data.clone(),
//                 value: vec![],
//                 key: vec![],
//                 node_type: EthTrieNodeType::Empty,
//             }
//         } else {
//             panic!("Unreachable code")
//         }
//     }
// }

//#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct EthProver {
    bridge_smart_contract: String,
}

//#[near_bindgen]
impl EthProver {
    pub fn init(bridge_smart_contract: String) -> Self {
        Self {
            bridge_smart_contract
        }
    }

    fn matchingNibbleLength(a: Vec<u8>, b: Vec<u8>) -> usize {
        a.iter().zip(b.iter()).take_while(|(i,j)| i == j).count()
    }

    /// https://github.com/slockit/in3/blob/master/src/util/merkleProof.ts
    pub fn verify_log_entry(
        &self,
        log_entry_data: Vec<u8>,
        receipt_data: Vec<u8>,
        header_data: Vec<u8>,
        proof: Vec<H256>,
        receipt_index_hash: H256,
    ) -> bool {
        let log_entry: LogEntry = rlp::decode(log_entry_data.as_slice()).unwrap();
        let receipt: Receipt = rlp::decode(receipt_data.as_slice()).unwrap();
        let header: BlockHeader = rlp::decode(header_data.as_slice()).unwrap();

        // create the nibbles to iterate over the path
        let mut key = bytesToNibbles((receipt_index_hash.0).0.to_vec());

        // start with the root-Hash
        let mut wantedHash = header.receipts_root;
        let mut lastNode: Option<EthTrieNode> = Option::None;

        // iterate through the nodes starting at root
        for i in 0..proof.len() {
            let p = proof[i];
            let hash = near_keccak256(&(p.0).0);

            // verify the hash of the node
            assert_eq!(wantedHash, hash.into());

            // create the node
            let node: EthTrieNode = rlp::decode(&(p.0).0).unwrap();
            //node.raw = (p.0).0.to_vec();
            lastNode = Option::Some(node.clone());

            match node.node_type {
                EthTrieNodeType::Empty => {
                    return i == 0 && receipt_data.len() == 0;
                }
                EthTrieNodeType::Branch => {
                    // we reached the end of the path
                    if key.len() == 0 {
                        return i == proof.len() - 1 && node.value == receipt_data;
                    }

                    // find the childHash
                    let childHash = node.raw[key[0] as usize].clone();
                    // remove the first item
                    key = key.iter().skip(1).cloned().collect();

                    if childHash.len() == 2 {
                        let embeddedNode: EthTrieNode = rlp::decode(childHash.as_slice()).unwrap();
                        if i != proof.len() - 1 {
                            panic!("Additional nodes at end of proof (embeddedNode)");
                        }

                        if Self::matchingNibbleLength(embeddedNode.key, key) != embeddedNode.key.len() {
                            panic!("Key length does not match with the proof one (embeddedNode)");
                        }

                        key = key.iter().skip(embeddedNode.key.len()).cloned().collect();
                        if key.len() != 0 {
                            panic!("Key does not match with the proof one (embeddedNode)");
                        }

                        // all is fine we return the value
                        return true;
                    } else {
                        wantedHash = childHash;
                    }
                },
                EthTrieNodeType::Leaf | EthTrieNodeType::Extension => {
                    let val = node.value;

                    // if the relativeKey in the leaf does not math our rest key, we throw!
                    if Self::matchingNibbleLength(node.key, key) != node.key.len() {
                        // so we have a wrong leaf here, if we actually expected this node to not exist,
                        // the last node in this path may be a different leaf or a branch with a empty hash
                        if key.len() == node.key.len() && i == proof.len() - 1 && header.receipts_root == Option::None {
                            return val;
                        }

                        panic!("Key does not match with the proof one (extention|leaf)");
                    }

                    // remove the items
                    key = key.iter().skip(node.key.len()).cloned().collect();

                    if key.len() == 0 {
                        if i != proof.len() - 1 {
                            panic!("Additional nodes at end of proof (extention|leaf)");
                        }

                        // if we are expecting a value we need to check
                        if header.receipts_root && header.receipts_root.compare(val) {
                            panic!("The proven value was expected to be {:?} but is {:?}", header.receipts_root, val);
                        }

                        // if we are proven a value which shouldn't exist this must throw an error
                        if header.receipts_root == Option::None {
                            panic!("The value shouldn't exist, but is {:?}", val);
                        }

                        return val;
                    } else {
                        // we continue with the hash 
                        wantedHash = val;
                    }
                },
            }
        }

        // if we expected this to be null and there is not further node since wantedHash is empty or we had a extension as last element, than it is ok not to find leafs
        if header.receipts_root == Option::None && (lastNode == Option::None || lastNode.node_type == EthTrieNodeType::Extension || wantedHash.len() == 0) {
            return false
        }

        // we reached the end of the proof, but not of the path
        panic!("Unexpected end of proof");
    }
}
