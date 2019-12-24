use std::io::{Error, Read, Write};
use rlp::{Rlp, RlpStream, DecoderError as RlpDecoderError, Decodable as RlpDecodable, Encodable as RlpEncodable};
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use ethereum_types;
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::{near_bindgen};

macro_rules! arr_declare_wrapper_and_serde {
    ($name: ident, $len: expr) => {
        #[near_bindgen]
        #[derive(Default, Clone, Copy, PartialEq, Debug)]
        pub struct $name(pub ethereum_types::$name);

        impl BorshSerialize for $name {
            #[inline]
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                writer.write_all(&(self.0).0)
            }
        }

        impl BorshDeserialize for $name {
            #[inline]
            fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
                let mut data = [0u8; $len];
                reader.read_exact(&mut data)?;
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

macro_rules! uint_declare_wrapper_and_serde {
    ($name: ident, $len: expr) => {
        #[near_bindgen]
        #[derive(Default, Clone, Copy, PartialEq, Debug)]
        pub struct $name(pub ethereum_types::$name);

        impl BorshSerialize for $name {
            #[inline]
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                for i in 0..$len {
                    u64::serialize(&(self.0).0[i], writer)?;
                }
                Ok(())
            }
        }

        impl BorshDeserialize for $name {
            #[inline]
            fn deserialize<R: Read>(reader: &mut R) -> Result<Self, Error> {
                let mut data = [0u64; $len];
                for i in 0..$len {
                    data[i] = u64::deserialize(reader)?;
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

arr_declare_wrapper_and_serde!(H64, 8);
arr_declare_wrapper_and_serde!(H128, 16);
arr_declare_wrapper_and_serde!(H160, 20);
arr_declare_wrapper_and_serde!(H256, 32);
arr_declare_wrapper_and_serde!(H512, 64);
arr_declare_wrapper_and_serde!(H520, 65);
arr_declare_wrapper_and_serde!(Bloom, 256);

uint_declare_wrapper_and_serde!(U64, 1);
uint_declare_wrapper_and_serde!(U128, 2);
uint_declare_wrapper_and_serde!(U256, 4);

pub type Address = H160;
pub type Secret = H256;
pub type Public = H512;
pub type Signature = H520;

pub fn keccak256(data: &[u8]) -> H256 {
    let mut hasher = Sha3::keccak256();
    hasher.input(data);

    let mut buffer = [0u8; 32];
    hasher.result(&mut buffer);
    H256(ethereum_types::H256(buffer))
}
