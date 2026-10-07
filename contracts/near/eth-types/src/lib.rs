use borsh::io::Read;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Rem, RemAssign, Sub,
    SubAssign,
};
use rlp::{
    Decodable as RlpDecodable, DecoderError as RlpDecoderError, DecoderError,
    Encodable as RlpEncodable, Rlp, RlpStream,
};
use rlp_derive::RlpDecodable as RlpDecodableDerive;
#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use sha3::{Digest, Keccak256, Keccak512};

use std::io::{Error, Write};
#[cfg(feature = "eth2")]
use tree_hash::PackedEncoding;

#[cfg(feature = "eth2")]
pub mod eth2;
#[macro_use]
pub mod macros;

arr_ethereum_types_wrapper_impl_borsh_serde_ssz!(H64, 8);
arr_ethereum_types_wrapper_impl_borsh_serde_ssz!(H128, 16);
arr_ethereum_types_wrapper_impl_borsh_serde_ssz!(H160, 20);
arr_ethereum_types_wrapper_impl_borsh_serde_ssz!(H256, 32);
arr_ethereum_types_wrapper_impl_borsh_serde_ssz!(H512, 64);
arr_ethereum_types_wrapper_impl_borsh_serde_ssz!(H520, 65);
arr_ethereum_types_wrapper_impl_borsh_serde_ssz!(Bloom, 256);

macro_rules! uint_declare_wrapper_and_serde {
    ($name: ident, $len: expr) => {
        #[derive(
            Default,
            Clone,
            Copy,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Debug,
            Add,
            Sub,
            Mul,
            Div,
            Rem,
            AddAssign,
            SubAssign,
            MulAssign,
            DivAssign,
            RemAssign,
            Display,
            From,
            Into,
        )]
        #[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
        pub struct $name(pub ethereum_types::$name);

        // For uint types (U64, U128, U256)
        impl borsh::BorshSchema for $name {
            fn declaration() -> String {
                stringify!($name).to_string()
            }

            fn add_definitions_recursively(
                definitions: &mut std::collections::BTreeMap<String, borsh::schema::Definition>,
            ) {
                // Define as a sequence of u64 values
                let definition = borsh::schema::Definition::Sequence {
                    length_width: borsh::schema::Definition::ARRAY_LENGTH_WIDTH, // 0 for fixed-size
                    length_range: $len as u64..=$len as u64,                     // Fixed length
                    elements: u64::declaration(),
                };

                borsh::schema::add_definition(Self::declaration(), definition, definitions);
                u64::add_definitions_recursively(definitions);
            }
        }

        impl BorshSerialize for $name {
            #[inline]
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                for i in 0..$len {
                    BorshSerialize::serialize(&(self.0).0[i], writer)?;
                }
                Ok(())
            }
        }

        impl BorshDeserialize for $name {
            #[inline]
            fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
                let mut data = [0u64; $len];
                for i in 0..$len {
                    data[i] = u64::deserialize_reader(reader)?;
                }
                Ok($name(ethereum_types::$name(data)))
            }
        }

        impl RlpEncodable for $name {
            fn rlp_append(&self, s: &mut RlpStream) {
                <ethereum_types::$name>::rlp_append(&self.0, s);
            }
        }

        impl RlpDecodable for $name {
            fn decode(rlp: &Rlp) -> Result<Self, RlpDecoderError> {
                Ok($name(<ethereum_types::$name>::decode(rlp)?))
            }
        }

        impl From<Vec<u8>> for $name {
            fn from(bytes: Vec<u8>) -> Self {
                Self(ethereum_types::$name::from_big_endian(&bytes))
            }
        }

        impl From<&[u8]> for $name {
            fn from(bytes: &[u8]) -> Self {
                Self(ethereum_types::$name::from_big_endian(bytes))
            }
        }

        impl $name {
            /// Create from big-endian bytes
            pub fn from_bytes(bytes: &[u8]) -> Self {
                Self(ethereum_types::$name::from_big_endian(bytes))
            }

            /// Convert to big-endian byte vector
            pub fn to_bytes(&self) -> Vec<u8> {
                let mut bytes = vec![0u8; $len * 8];
                self.0.to_big_endian(&mut bytes);
                bytes
            }
        }

        impl tree_hash::TreeHash for $name {
            fn tree_hash_type() -> tree_hash::TreeHashType {
                tree_hash::TreeHashType::Basic
            }

            fn tree_hash_packed_encoding(&self) -> PackedEncoding {
                let mut bytes = [0u8; 32];
                self.0.to_little_endian(&mut bytes);
                PackedEncoding::from(bytes)
            }

            fn tree_hash_packing_factor() -> usize {
                1
            }

            fn tree_hash_root(&self) -> tree_hash::Hash256 {
                let mut bytes = [0u8; 32];
                self.0.to_little_endian(&mut bytes);
                tree_hash::Hash256::from(bytes)
            }
        }
    };
}

uint_declare_wrapper_and_serde!(U64, 1);
uint_declare_wrapper_and_serde!(U128, 2);
uint_declare_wrapper_and_serde!(U256, 4);

pub type Address = H160;
pub type Secret = H256;
pub type Public = H512;
pub type Signature = H520;

// Block Header

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, BorshSchema)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
#[cfg_attr(not(target_arch = "wasm32"), serde(rename_all = "camelCase"))]
pub struct BlockHeader {
    pub parent_hash: H256,
    pub sha3_uncles: H256,
    pub miner: Address,
    pub state_root: H256,
    pub transactions_root: H256,
    pub receipts_root: H256,
    pub logs_bloom: Bloom,
    pub difficulty: U256,
    #[cfg_attr(
        all(feature = "eth2", not(target_arch = "wasm32")),
        serde(with = "serde_utils::u64_hex_be")
    )]
    pub number: u64,
    pub gas_limit: U256,
    pub gas_used: U256,
    #[cfg_attr(
        all(feature = "eth2", not(target_arch = "wasm32")),
        serde(with = "serde_utils::u64_hex_be")
    )]
    pub timestamp: u64,
    #[cfg_attr(
        all(feature = "eth2", not(target_arch = "wasm32")),
        serde(with = "serde_utils::hex_vec")
    )]
    pub extra_data: Vec<u8>,
    pub mix_hash: H256,
    pub nonce: H64,
    #[cfg_attr(
        all(feature = "eth2", not(target_arch = "wasm32")),
        serde(with = "u64_hex_be_option")
    )]
    pub base_fee_per_gas: Option<u64>,
    pub withdrawals_root: Option<H256>,
    #[cfg_attr(
        all(feature = "eth2", not(target_arch = "wasm32")),
        serde(with = "u64_hex_be_option"),
        serde(default)
    )]
    pub blob_gas_used: Option<u64>,
    #[cfg_attr(
        all(feature = "eth2", not(target_arch = "wasm32")),
        serde(with = "u64_hex_be_option"),
        serde(default)
    )]
    pub excess_blob_gas: Option<u64>,
    #[cfg_attr(all(feature = "eth2", not(target_arch = "wasm32")), serde(default))]
    pub parent_beacon_block_root: Option<H256>,
    #[cfg_attr(all(feature = "eth2", not(target_arch = "wasm32")), serde(default))]
    pub requests_hash: Option<H256>,
    // Optional fields introduced in Amsterdam fork
    // EIP-7928: Block-Level Access Lists
    #[cfg_attr(all(feature = "eth2", not(target_arch = "wasm32")), serde(default))]
    pub block_access_list_hash: Option<H256>,
    // EIP-7843: SLOTNUM opcode
    #[cfg_attr(
        all(feature = "eth2", not(target_arch = "wasm32")),
        serde(with = "u64_hex_be_option"),
        serde(default)
    )]
    pub slot_number: Option<u64>,

    pub hash: Option<H256>,
    pub partial_hash: Option<H256>,
}
#[cfg(all(feature = "eth2", not(target_arch = "wasm32")))]
pub mod u64_hex_be_option {
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Some(serde_utils::u64_hex_be::deserialize(deserializer)?))
    }

    pub fn serialize<S>(num: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(num) = num {
            serde_utils::u64_hex_be::serialize(num, serializer)
        } else {
            serializer.serialize_none()
        }
    }
}

impl BlockHeader {
    pub fn extra_data(&self) -> H256 {
        let mut data = [0u8; 32];
        data.copy_from_slice(self.extra_data.as_slice());
        H256(data.into())
    }

    fn stream_rlp(&self, stream: &mut RlpStream, partial: bool) {
        let mut list_size = 13;
        if !partial {
            list_size += 2;
        }
        if self.base_fee_per_gas.is_some() {
            list_size += 1;
        }
        if self.withdrawals_root.is_some() {
            list_size += 1;
        }
        if self.blob_gas_used.is_some() {
            list_size += 1;
        }
        if self.excess_blob_gas.is_some() {
            list_size += 1;
        }
        if self.parent_beacon_block_root.is_some() {
            list_size += 1;
        }
        if self.requests_hash.is_some() {
            list_size += 1;
        }
        if self.block_access_list_hash.is_some() {
            list_size += 1;
        }
        if self.slot_number.is_some() {
            list_size += 1;
        }

        stream.begin_list(list_size);

        stream.append(&self.parent_hash);
        stream.append(&self.sha3_uncles);
        stream.append(&self.miner);
        stream.append(&self.state_root);
        stream.append(&self.transactions_root);
        stream.append(&self.receipts_root);
        stream.append(&self.logs_bloom);
        stream.append(&self.difficulty);
        stream.append(&self.number);
        stream.append(&self.gas_limit);
        stream.append(&self.gas_used);
        stream.append(&self.timestamp);
        stream.append(&self.extra_data);

        if !partial {
            stream.append(&self.mix_hash);
            stream.append(&self.nonce);
        }

        if let Some(base_fee_per_gas) = &self.base_fee_per_gas {
            stream.append(base_fee_per_gas);
        }

        if let Some(withdrawals_root) = &self.withdrawals_root {
            stream.append(withdrawals_root);
        }

        if let Some(blob_gas_used) = &self.blob_gas_used {
            stream.append(blob_gas_used);
        }

        if let Some(excess_blob_gas) = &self.excess_blob_gas {
            stream.append(excess_blob_gas);
        }

        if let Some(parent_beacon_block_root) = &self.parent_beacon_block_root {
            stream.append(parent_beacon_block_root);
        }

        if let Some(requests_hash) = &self.requests_hash {
            stream.append(requests_hash);
        }

        if let Some(block_access_list_hash) = &self.block_access_list_hash {
            stream.append(block_access_list_hash);
        }

        if let Some(slot_number) = &self.slot_number {
            stream.append(slot_number);
        }
    }

    pub fn calculate_hash(&self) -> H256 {
        near_keccak256({
            let mut stream = RlpStream::new();
            self.stream_rlp(&mut stream, false);
            &stream.out()[..]
        })
        .into()
    }
}

impl RlpEncodable for BlockHeader {
    fn rlp_append(&self, stream: &mut RlpStream) {
        self.stream_rlp(stream, false);
    }
}

impl RlpDecodable for BlockHeader {
    fn decode(serialized: &Rlp) -> Result<Self, RlpDecoderError> {
        let mut block_header = BlockHeader {
            parent_hash: serialized.val_at(0)?,
            sha3_uncles: serialized.val_at(1)?,
            miner: serialized.val_at(2)?,
            state_root: serialized.val_at(3)?,
            transactions_root: serialized.val_at(4)?,
            receipts_root: serialized.val_at(5)?,
            logs_bloom: serialized.val_at(6)?,
            difficulty: serialized.val_at(7)?,
            number: serialized.val_at(8)?,
            gas_limit: serialized.val_at(9)?,
            gas_used: serialized.val_at(10)?,
            timestamp: serialized.val_at(11)?,
            extra_data: serialized.val_at(12)?,
            mix_hash: serialized.val_at(13)?,
            nonce: serialized.val_at(14)?,
            base_fee_per_gas: serialized.val_at(15).ok(),
            withdrawals_root: serialized.val_at(16).ok(),
            blob_gas_used: serialized.val_at(17).ok(),
            excess_blob_gas: serialized.val_at(18).ok(),
            parent_beacon_block_root: serialized.val_at(19).ok(),
            requests_hash: serialized.val_at(20).ok(),
            block_access_list_hash: serialized.val_at(21).ok(),
            slot_number: serialized.val_at(22).ok(),
            hash: None,
            partial_hash: None,
        };

        block_header.hash = Some(
            near_keccak256({
                let mut stream = RlpStream::new();
                block_header.stream_rlp(&mut stream, false);
                &stream.out()[..]
            })
            .into(),
        );

        if block_header.hash.unwrap() != near_keccak256(serialized.as_raw()).into() {
            return Err(RlpDecoderError::RlpInconsistentLengthAndData);
        }

        block_header.partial_hash = Some(
            near_keccak256({
                let mut stream = RlpStream::new();
                block_header.stream_rlp(&mut stream, true);
                &stream.out()[..]
            })
            .into(),
        );

        Ok(block_header)
    }
}

// Log

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub address: Address,
    pub topics: Vec<H256>,
    pub data: Vec<u8>,
}

impl rlp::Decodable for LogEntry {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let result = LogEntry {
            address: rlp.val_at(0usize)?,
            topics: rlp.list_at(1usize)?,
            data: rlp.val_at(2usize)?,
        };
        Ok(result)
    }
}

impl rlp::Encodable for LogEntry {
    fn rlp_append(&self, stream: &mut rlp::RlpStream) {
        stream.begin_list(3usize);
        stream.append(&self.address);
        stream.append_list::<H256, _>(&self.topics);
        stream.append(&self.data);
    }
}

// Receipt Header

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub status: bool,
    pub gas_used: U256,
    pub logs_bloom: Bloom,
    pub logs: Vec<LogEntry>,
}

impl rlp::Decodable for Receipt {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let mut view = rlp.as_raw();

        // https://eips.ethereum.org/EIPS/eip-2718
        if let Some(&byte) = view.first() {
            // https://eips.ethereum.org/EIPS/eip-2718#receipts
            // If the first byte is between 0 and 0x7f it is an envelop receipt
            if byte <= 0x7f {
                view = &view[1..];
            }
        }

        rlp::decode::<RlpDeriveReceipt>(view).map(Into::into)
    }
}

#[derive(RlpDecodableDerive)]
pub struct RlpDeriveReceipt {
    pub status: bool,
    pub gas_used: U256,
    pub logs_bloom: Bloom,
    pub logs: Vec<LogEntry>,
}

impl From<RlpDeriveReceipt> for Receipt {
    fn from(receipt: RlpDeriveReceipt) -> Self {
        Self {
            status: receipt.status,
            gas_used: receipt.gas_used,
            logs_bloom: receipt.logs_bloom,
            logs: receipt.logs,
        }
    }
}

pub fn near_sha256(data: &[u8]) -> [u8; 32] {
    let mut buffer = [0u8; 32];
    buffer.copy_from_slice(near_sdk::env::sha256(data).as_slice());
    buffer
}

pub fn near_keccak256(data: &[u8]) -> [u8; 32] {
    #[cfg(target_arch = "wasm32")]
    {
        near_sdk::env::keccak256(data).try_into().unwrap()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Keccak256::digest(data).try_into().unwrap()
    }
}

pub fn near_keccak512(data: &[u8]) -> [u8; 64] {
    #[cfg(target_arch = "wasm32")]
    {
        near_sdk::env::keccak512(data).try_into().unwrap()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Keccak512::digest(data).try_into().unwrap()
    }
}

#[cfg(all(test, feature = "eth2"))]
mod tests {
    use super::*;

    // Sepolia block 11863284 (Amsterdam)
    const AMSTERDAM_BLOCK: &str = r#"{
        "hash": "0x5bacafa052d4e775218db0ad3622b8c6c860f217707ed61cd377703954a178bc",
        "parentHash": "0xca4669bf950092c8a99e17b1b753d4f51e3b462281124540fb6264aed7c0e49b",
        "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
        "miner": "0xa214fd87626b00349f457154382c6ec6a2659d1c",
        "stateRoot": "0x05a66d79359586093a08c4f7a4b4ca572f190dbc73e63d7a3d4eac1ea99a5d4e",
        "transactionsRoot": "0x8c09bc47a79998859fecd7068c10231670cae0428df79cd60e1b1434fb0be42b",
        "receiptsRoot": "0x357abc171b1b5fee8b6fcef541a9dea10057d8e825bdc8cc0ca9936b7c62df00",
        "logsBloom": "0x1c310151920d843a5fcb150cdc149c400904815b3170866b8b9028546222054e78d84c049b20f39b12e22daa680040392e119964a6aa4f031f319062fa7e324f7504f9e10e62e114a78010ded23180b16631c2faf1642c51a0e98cfac8a324091f104976beb0a81b360942f880380c4c6a8102c03fef0c6511c65ed3c60a2058a0863ad109506b4c11eb85943c16ed1581c0adc5201b912b310692ff84da0a810a0a41825ab82e511a941142f46ac40e264235012d8c8065a128328486b14b91ec3d89ce850103d840698d242882010272fe9082950c013c052991120024fbcba370074f4620e8aed822bb01e040b48c39a214088da4035b9fbb8f228391ca18",
        "difficulty": "0x0",
        "number": "0xb504f4",
        "gasLimit": "0xbebc200",
        "gasUsed": "0x4435a24",
        "timestamp": "0x6ac648d4",
        "extraData": "0x",
        "mixHash": "0x85619a4ba9e269a43022080d2792b09ebc1e265eeb2b64df22108aebc62c33ab",
        "nonce": "0x0000000000000000",
        "baseFeePerGas": "0x25",
        "withdrawalsRoot": "0x2b322cdf5d2600ea4999d8a7942a90e11af054091d73d805e61eff536e1ef68d",
        "blobGasUsed": "0x180000",
        "excessBlobGas": "0xc8a9a6",
        "parentBeaconBlockRoot": "0xcc8356e6db134a5d3a40e80bacfa9889c147e179f82dbc0140ed873ce6765e14",
        "requestsHash": "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "blockAccessListHash": "0xe9341ade1c03edb03793d6ecb8b6bb36b1e4bad6617e594e643dde12c14cf3fc",
        "slotNumber": "0xac7b9f"
    }"#;

    // Sepolia block 11799104 (Prague, before Amsterdam)
    const PRAGUE_BLOCK: &str = r#"{
        "hash": "0xc8b2599c9fa9927887dbc3687f08a0236e6b94f0832285418d0a656fc8bb5239",
        "parentHash": "0x2dab7b273381421669c8f274d10080afe0dfafa28d22d4fef0a061da96dc3a2e",
        "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
        "miner": "0x0000006916a87b82333f4245046623b23794c65c",
        "stateRoot": "0x9c26c3c98a6ae221fc97dcbc3971d3398e7ecb3e3bc1083e2952c6f9ac4bf1b0",
        "transactionsRoot": "0x197293b65d28aa2024093b62d36984f6a3d43441dea883e90d028d6bb6ee0ecf",
        "receiptsRoot": "0x02c3a2f1b3fc5200c1b407f40374bb1be8d1c5c3944071523f23185a6322efa4",
        "logsBloom": "0xceb21564501979740b01b158fc7616485b84974de1a5f81e53f082b462378dfbeddc5d2ad0aa7f14d02eaf1f3829ff308253c4a7f6fa3d41924a9a0d6fbd2726d3471dc1d6811718669014dfc6381f6c07d362acb1e5c556a06b37acb8e0b5c17fb21bb2beb92912c004cda37d1e4de7fdf216da267e65383c30dd70c42f3251e9c67847b6deca5951c88d1ba6a2401145ec4fc9307cd4bb59f7346644b66a414ab240221b14410e9944c7700c28edaeaf475da80a8d9c19a1e00ac02e3809abd6ec41e20f1af4c99f17f42448129928deb2b2caaf3e487b7859576f61276eddf1da6d1a46b4e2b32a1a14e26f3fc98f6387831c89a8156acbabba769fd96a2b",
        "difficulty": "0x0",
        "number": "0xb40e40",
        "gasLimit": "0x3938700",
        "gasUsed": "0x143de1d",
        "timestamp": "0x6aba4b44",
        "extraData": "0xd883011008846765746888676f312e32342e30856c696e7578",
        "mixHash": "0xe5f776a0c6749f6e9ed1c7748b708bf11f77852974f6444180d4045ff80b8abc",
        "nonce": "0x0000000000000000",
        "baseFeePerGas": "0x411c308f",
        "withdrawalsRoot": "0x00ada0ab236a3c94083ec9cc6b0ad43d6fd63319896014a26bedd7ec40034076",
        "blobGasUsed": "0x1e0000",
        "excessBlobGas": "0xc81fb69",
        "parentBeaconBlockRoot": "0x539f965d9bfbfcbbb04a4b52c8ed95af4858a55f7b36a9cdf7b7ed752b987810",
        "requestsHash": "0xe3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    }"#;

    fn parse_block(json: &str) -> (BlockHeader, H256) {
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let expected_hash: H256 = serde_json::from_value(value["hash"].clone()).unwrap();
        let header: BlockHeader = serde_json::from_value(value).unwrap();
        (header, expected_hash)
    }

    fn assert_rlp_roundtrip(header: &BlockHeader, expected_hash: H256) {
        let encoded = rlp::encode(header);
        let decoded: BlockHeader = rlp::decode(&encoded).unwrap();
        assert_eq!(decoded.hash, Some(expected_hash));
        assert_eq!(decoded.requests_hash, header.requests_hash);
        assert_eq!(
            decoded.block_access_list_hash,
            header.block_access_list_hash
        );
        assert_eq!(decoded.slot_number, header.slot_number);
    }

    #[test]
    fn test_amsterdam_block_header_hash() {
        let (header, expected_hash) = parse_block(AMSTERDAM_BLOCK);
        assert!(header.block_access_list_hash.is_some());
        assert_eq!(header.slot_number, Some(0xac7b9f));
        assert_eq!(header.calculate_hash(), expected_hash);
        assert_rlp_roundtrip(&header, expected_hash);
    }

    #[test]
    fn test_prague_block_header_hash() {
        let (header, expected_hash) = parse_block(PRAGUE_BLOCK);
        assert!(header.requests_hash.is_some());
        assert!(header.block_access_list_hash.is_none());
        assert!(header.slot_number.is_none());
        assert_eq!(header.calculate_hash(), expected_hash);
        assert_rlp_roundtrip(&header, expected_hash);
    }

    #[test]
    fn test_amsterdam_block_header_borsh_roundtrip() {
        let (header, expected_hash) = parse_block(AMSTERDAM_BLOCK);
        let decoded = BlockHeader::try_from_slice(&borsh::to_vec(&header).unwrap()).unwrap();
        assert_eq!(decoded.calculate_hash(), expected_hash);
    }
}
