use borsh::{BorshDeserialize, BorshSerialize};
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
use tree_hash::{PackedEncoding, TreeHash, TreeHashType};

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

#[cfg(feature = "eth2")]
impl TreeHash for H256 {
    fn tree_hash_type() -> TreeHashType {
        TreeHashType::Vector
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        PackedEncoding::from_slice(self.0.as_bytes())
    }

    fn tree_hash_packing_factor() -> usize {
        1
    }

    fn tree_hash_root(&self) -> tree_hash::Hash256 {
        (*self).0
    }
}

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
            fn deserialize(buf: &mut &[u8]) -> Result<Self, Error> {
                let mut data = [0u64; $len];
                for i in 0..$len {
                    data[i] = borsh::de::BorshDeserialize::deserialize(buf)?;
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

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct BlockHeader {
    pub parent_hash: H256,
    pub uncles_hash: H256,
    pub author: Address,
    pub state_root: H256,
    pub transactions_root: H256,
    pub receipts_root: H256,
    pub log_bloom: Bloom,
    pub difficulty: U256,
    #[cfg_attr(
        all(feature = "eth2", not(target_arch = "wasm32")),
        serde(with = "eth2_serde_utils::u64_hex_be")
    )]
    pub number: u64,
    pub gas_limit: U256,
    pub gas_used: U256,
    #[cfg_attr(
        all(feature = "eth2", not(target_arch = "wasm32")),
        serde(with = "eth2_serde_utils::u64_hex_be")
    )]
    pub timestamp: u64,
    #[cfg_attr(
        all(feature = "eth2", not(target_arch = "wasm32")),
        serde(with = "eth2_serde_utils::hex_vec")
    )]
    pub extra_data: Vec<u8>,
    pub mix_hash: H256,
    pub nonce: H64,
    #[cfg_attr(
        all(feature = "eth2", not(target_arch = "wasm32")),
        serde(deserialize_with = "u64_hex_be_option")
    )]
    pub base_fee_per_gas: Option<u64>,
    pub withdrawals_root: Option<H256>,

    pub hash: Option<H256>,
    pub partial_hash: Option<H256>,
}

#[cfg(all(feature = "eth2", not(target_arch = "wasm32")))]
fn u64_hex_be_option<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(eth2_serde_utils::u64_hex_be::deserialize(
        deserializer,
    )?))
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

        stream.begin_list(list_size);

        stream.append(&self.parent_hash);
        stream.append(&self.uncles_hash);
        stream.append(&self.author);
        stream.append(&self.state_root);
        stream.append(&self.transactions_root);
        stream.append(&self.receipts_root);
        stream.append(&self.log_bloom);
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
            uncles_hash: serialized.val_at(1)?,
            author: serialized.val_at(2)?,
            state_root: serialized.val_at(3)?,
            transactions_root: serialized.val_at(4)?,
            receipts_root: serialized.val_at(5)?,
            log_bloom: serialized.val_at(6)?,
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
    pub log_bloom: Bloom,
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
    pub log_bloom: Bloom,
    pub logs: Vec<LogEntry>,
}

impl From<RlpDeriveReceipt> for Receipt {
    fn from(receipt: RlpDeriveReceipt) -> Self {
        Self {
            status: receipt.status,
            gas_used: receipt.gas_used,
            log_bloom: receipt.log_bloom,
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
