use std::io::{Error, Read, Write};
use rlp::{Rlp, RlpStream, DecoderError as RlpDecoderError, Decodable as RlpDecodable, Encodable as RlpEncodable};
use rlp_derive::{RlpEncodable as RlpEncodableDerive, RlpDecodable as RlpDecodableDerive};
use ethereum_types;
#[cfg(not(target_arch = "wasm32"))]
use serde::{Serialize, Deserialize};
use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::{Add, Sub, Mul, Div, Rem, AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, Display, From, Into};

macro_rules! arr_declare_wrapper_and_serde {
    ($name: ident, $len: expr) => {
        #[derive(Default, Clone, Copy, Eq, PartialEq, Debug, Display, From, Into)]
        #[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
        pub struct $name(pub ethereum_types::$name);

        impl From<&[u8; $len]> for $name {
            fn from(item: &[u8; $len]) -> Self {
                $name(item.into())
            }
        }

        impl From<[u8; $len]> for $name {
            fn from(item: [u8; $len]) -> Self {
                (&item).into()
            }
        }

        impl From<&Vec<u8>> for $name {
            fn from(item: &Vec<u8>) -> Self {
                let mut data = [0u8; $len];
                for i in 0..item.len() {
                    data[$len - 1 - i] = item[item.len() - 1 - i];
                }
                $name(data.into())
            }
        }

        impl From<Vec<u8>> for $name {
            fn from(item: Vec<u8>) -> Self {
                (&item).into()
            }
        }

        impl From<&[u8]> for $name {
            fn from(item: &[u8]) -> Self {
                item.to_vec().into()
            }
        }

        impl BorshSerialize for $name {
            #[inline]
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                writer.write_all(&(self.0).0)?;
                Ok(())
            }
        }

        impl BorshDeserialize for $name {
            #[inline]
            fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
                let mut data = [0u8; $len];
                reader.read_exact(&mut data)?;
                Ok($name(data.into()))
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
    }
}

arr_declare_wrapper_and_serde!(H64, 8);
arr_declare_wrapper_and_serde!(H128, 16);
arr_declare_wrapper_and_serde!(H160, 20);
arr_declare_wrapper_and_serde!(H256, 32);
arr_declare_wrapper_and_serde!(H512, 64);
arr_declare_wrapper_and_serde!(H520, 65);
arr_declare_wrapper_and_serde!(Bloom, 256);

macro_rules! uint_declare_wrapper_and_serde {
    ($name: ident, $len: expr) => {
        #[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Add, Sub, Mul, Div, Rem, AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, Display, From, Into)]
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
            fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
                let mut data = [0u64; $len];
                for i in 0..$len {
                    data[i] = borsh::de::BorshDeserialize::deserialize(reader)?;
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
    }
}

uint_declare_wrapper_and_serde!(U64, 1);
uint_declare_wrapper_and_serde!(U128, 2);
uint_declare_wrapper_and_serde!(U256, 4);

pub type Address = H160;
pub type Secret = H256;
pub type Public = H512;
pub type Signature = H520;

// Block Header

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
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
    pub number: u64,
    pub gas_limit: U256,
    pub gas_used: U256,
    pub timestamp: u64,
    pub extra_data: Vec<u8>,
    pub mix_hash: H256,
    pub nonce: H64,

    pub hash: Option<H256>,
    pub partial_hash: Option<H256>,
}

impl BlockHeader {
    pub fn extra_data(&self) -> H256 {
        let mut data = [0u8; 32];
        data.copy_from_slice(self.extra_data.as_slice());
        H256(data.into())
    }

    fn stream_rlp(&self, stream: &mut RlpStream, partial: bool) {
        stream.begin_list(13 + if !partial { 2 } else { 0 });

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
            hash: Some(near_keccak256(serialized.as_raw()).into()),
            partial_hash: None,
        };

        block_header.partial_hash = Some(near_keccak256({
            let mut stream = RlpStream::new();
            block_header.stream_rlp(&mut stream, true);
            stream.out().as_slice()
        }).into());

        Ok(block_header)
    }
}

// Log

#[derive(Default, Debug, Clone, PartialEq, Eq, RlpEncodableDerive, RlpDecodableDerive)]
pub struct LogEntry {
	pub address: Address,
	pub topics: Vec<H256>,
	pub data: Vec<u8>,
}

// Receipt Header

#[derive(Debug, Clone, PartialEq, Eq, RlpEncodableDerive, RlpDecodableDerive)]
pub struct Receipt {
    pub state_root: H256,
    pub gas_used: U256,
	pub log_bloom: Bloom,
	pub logs: Vec<LogEntry>,
}

//

pub fn near_sha256(data: &[u8]) -> [u8; 32] {
    let mut buffer = [0u8; 32];
    buffer.copy_from_slice(&near_bindgen::env::sha256(data).as_slice());
    buffer
}

pub fn near_keccak256(data: &[u8]) -> [u8; 32] {
    let mut buffer = [0u8; 32];
    buffer.copy_from_slice(&near_bindgen::env::keccak256(data).as_slice());
    buffer
}

pub fn near_keccak512(data: &[u8]) -> [u8; 64] {
    let mut buffer = [0u8; 64];
    buffer.copy_from_slice(&near_bindgen::env::keccak512(data).as_slice());
    buffer
}
