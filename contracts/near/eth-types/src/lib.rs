use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Rem, RemAssign, Sub,
    SubAssign,
};
use ethereum_types;
use rlp::{
    Decodable as RlpDecodable, DecoderError as RlpDecoderError, DecoderError,
    Encodable as RlpEncodable, Rlp, RlpStream,
};
use rlp_derive::RlpDecodable as RlpDecodableDerive;
#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};
use std::io::{Error, Write};

const ERROR_UNEXPECTED_LENGTH_OF_INPUT: &str = "Unexpected length of input";

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
            fn deserialize(buf: &mut &[u8]) -> Result<Self, Error> {
                if buf.len() < $len {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        ERROR_UNEXPECTED_LENGTH_OF_INPUT,
                    ));
                }
                let mut data = [0u8; $len];
                data.copy_from_slice(&buf[..$len]);
                *buf = &buf[$len..];
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
    };
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

#[derive(Debug, Clone, BorshSerialize)]
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
    #[cfg(feature = "eip1559")]
    pub base_fee_per_gas: u64,

    pub hash: Option<H256>,
    pub partial_hash: Option<H256>,
}

#[derive(BorshDeserialize)]
pub struct BlockHeaderLondon {
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
    pub base_fee_per_gas: u64,

    pub hash: Option<H256>,
    pub partial_hash: Option<H256>,
}

impl From<BlockHeaderLondon> for BlockHeader {
    fn from(header: BlockHeaderLondon) -> Self {
        Self {
            parent_hash: header.parent_hash,
            uncles_hash: header.uncles_hash,
            author: header.author,
            state_root: header.state_root,
            transactions_root: header.transactions_root,
            receipts_root: header.receipts_root,
            log_bloom: header.log_bloom,
            difficulty: header.difficulty,
            number: header.number,
            gas_limit: header.gas_limit,
            gas_used: header.gas_used,
            timestamp: header.timestamp,
            extra_data: header.extra_data,
            mix_hash: header.mix_hash,
            nonce: header.nonce,
            #[cfg(feature = "eip1559")]
            base_fee_per_gas: header.base_fee_per_gas,
            hash: header.hash,
            partial_hash: header.partial_hash,
        }
    }
}

#[derive(BorshDeserialize)]
pub struct BlockHeaderPreLondon {
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

impl From<BlockHeaderPreLondon> for BlockHeader {
    fn from(header: BlockHeaderPreLondon) -> Self {
        Self {
            parent_hash: header.parent_hash,
            uncles_hash: header.uncles_hash,
            author: header.author,
            state_root: header.state_root,
            transactions_root: header.transactions_root,
            receipts_root: header.receipts_root,
            log_bloom: header.log_bloom,
            difficulty: header.difficulty,
            number: header.number,
            gas_limit: header.gas_limit,
            gas_used: header.gas_used,
            timestamp: header.timestamp,
            extra_data: header.extra_data,
            mix_hash: header.mix_hash,
            nonce: header.nonce,
            #[cfg(feature = "eip1559")]
            base_fee_per_gas: 7,
            hash: header.hash,
            partial_hash: header.partial_hash,
        }
    }
}

impl BorshDeserialize for BlockHeader {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        if let Ok(header) = BlockHeaderLondon::deserialize(buf) {
            Ok(header.into())
        } else {
            BlockHeaderPreLondon::deserialize(buf).map(Into::into)
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

        #[cfg(feature = "eip1559")]
        stream.append(&self.base_fee_per_gas);

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
            #[cfg(feature = "eip1559")]
            base_fee_per_gas: serialized.val_at(15)?,
            hash: Some(near_keccak256(serialized.as_raw()).into()),
            partial_hash: None,
        };

        block_header.partial_hash = Some(
            near_keccak256({
                let mut stream = RlpStream::new();
                block_header.stream_rlp(&mut stream, true);
                stream.out().as_slice()
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
    buffer.copy_from_slice(&near_sdk::env::sha256(data).as_slice());
    buffer
}

pub fn near_keccak256(data: &[u8]) -> [u8; 32] {
    let mut buffer = [0u8; 32];
    buffer.copy_from_slice(&near_sdk::env::keccak256(data).as_slice());
    buffer
}

pub fn near_keccak512(data: &[u8]) -> [u8; 64] {
    let mut buffer = [0u8; 64];
    buffer.copy_from_slice(&near_sdk::env::keccak512(data).as_slice());
    buffer
}

// Pol
#[cfg(feature = "pol")]
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct PolValidator {
    #[serde(default, rename = "ID")]
    pub id: u64,
    #[serde(default, rename = "signer")]
    pub address: Address,
    #[serde(default, rename = "power")]
    pub voting_power: u64,
    #[serde(default, rename = "accum")]
    pub proposer_priority: i64,
}

#[cfg(feature = "pol")]
impl PolValidator {
    fn stream_rlp(&self, stream: &mut RlpStream, partial: bool) {
        stream.begin_list(4);
        stream.append(&self.id);
        stream.append(&self.address);
        stream.append(&self.voting_power);
        stream.append(&self.proposer_priority.to_ne_bytes().to_vec());
    }
}

#[cfg(feature = "pol")]
impl RlpEncodable for PolValidator {
    fn rlp_append(&self, stream: &mut RlpStream) {
        self.stream_rlp(stream, false);
    }
}

#[cfg(feature = "pol")]
impl RlpDecodable for PolValidator {
    fn decode(serialized: &Rlp) -> Result<Self, RlpDecoderError> {
        let v: Vec<u8> = serialized.val_at(3)?;
        let mut r = [0u8; 8];
        r.copy_from_slice(&v[0..8]);
        Ok(PolValidator {
            id: serialized.val_at(0)?,
            address: serialized.val_at(1)?,
            voting_power: serialized.val_at(2)?,
            proposer_priority: i64::from_ne_bytes(r),
        })
    }
}

#[cfg(feature = "pol")]
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct PolValidatorSet {
    pub proposer: u64,
    pub validator_set: Vec<PolValidator>,
    pub total_voting_power: u64
}

#[cfg(feature = "pol")]
impl PolValidatorSet {
    fn stream_rlp(&self, stream: &mut RlpStream, partial: bool) {
        stream.begin_list(self.validator_set.len() + 3);
        stream.append(&self.proposer);
        stream.append(&self.total_voting_power);
        stream.append(&self.validator_set.len());
        for val in &self.validator_set {
            stream.append(val);
        }
    }
}

#[cfg(feature = "pol")]
impl RlpEncodable for PolValidatorSet {
    fn rlp_append(&self, stream: &mut RlpStream) {
        self.stream_rlp(stream, false);
    }
}

#[cfg(feature = "pol")]
impl RlpDecodable for PolValidatorSet {
    fn decode(serialized: &Rlp) -> Result<Self, RlpDecoderError> {
        let proposer: u64 = serialized.val_at(0)?;
        let total_voting_power: u64 = serialized.val_at(1)?;
        let validator_set_length: u64 = serialized.val_at(2)?;
        let mut validator_set: Vec<PolValidator> = Vec::new();

        for i in 3..validator_set_length {
            let val: Result<PolValidator, DecoderError> = serialized.val_at(i as usize);
            match val {
                Ok(v) => validator_set.push(v),
                Err(e) => return Err(e)
            }
        }

        Ok(PolValidatorSet {
            validator_set: validator_set,
            proposer: proposer,
            total_voting_power: total_voting_power,
        })
    }
}

// SealData struct used by pol to encode header to rlp and hash using keccak256.
#[cfg(feature = "pol")]
pub struct SealData<'s> {
    pub header: &'s BlockHeader,
}

#[cfg(feature = "pol")]
impl<'s> SealData<'s> {
    // hash using keccak256 an RLP encoded header.
    pub fn seal_hash(&self) -> [u8; 32] {
        near_keccak256(&self.rlp_bytes())
    }
}

// implement RlpEncodable for SealData.
#[cfg(feature = "pol")]
impl<'s> RlpEncodable for SealData<'s> {
    fn rlp_append(&self, stream: &mut RlpStream) {
        stream.begin_list(16);
        stream.append(&self.header.parent_hash);
        stream.append(&self.header.uncles_hash);
        stream.append(&self.header.author);
        stream.append(&self.header.state_root);
        stream.append(&self.header.transactions_root);
        stream.append(&self.header.receipts_root);
        stream.append(&self.header.log_bloom);
        stream.append(&self.header.difficulty);
        stream.append(&self.header.number);
        stream.append(&self.header.gas_limit);
        stream.append(&self.header.gas_used);
        stream.append(&self.header.timestamp);
        stream.append(&self.header.extra_data[0..&self.header.extra_data.len() - 65].to_vec());
        stream.append(&self.header.mix_hash);
        stream.append(&self.header.nonce);
    
        #[cfg(feature = "eip1559")]
        stream.append(&self.header.base_fee_per_gas);
    }

    fn rlp_bytes(&self) -> Vec<u8> {
        let mut s = RlpStream::new();
        self.rlp_append(&mut s);
        s.out()
    }
}
