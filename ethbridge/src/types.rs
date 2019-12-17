use std::io::{Error, Read, Write};
use rlp::{Rlp, RlpStream, DecoderError, Decodable, Encodable};
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use ethereum_types;
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::{near_bindgen};

macro_rules! impl_serde {
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
        
        impl Encodable for $name {
        	fn rlp_append(&self, s: &mut RlpStream) {
				<ethereum_types::$name>::rlp_append(&self.0, s);
            }
        }

		// TODO: fix
        impl Decodable for $name {
        	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
				Ok($name(<ethereum_types::$name>::decode(rlp)?))
        	}
        }

		// #[cfg(feature = "serialize")]
		// impl Serialize for $name {
		// 	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		// 		let mut slice = [0u8; 2 + 2 * $len * 8];
		// 		let mut bytes = [0u8; $len * 8];
		// 		self.to_big_endian(&mut bytes);
		// 		ethereum_types_serialize::serialize_uint(&mut slice, &bytes, serializer)
		// 	}
		// }

		// #[cfg(feature = "serialize")]
		// impl<'de> Deserialize<'de> for $name {
		// 	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		// 		let mut bytes = [0u8; $len * 8];
		// 		let wrote = ethereum_types_serialize::deserialize_check_len(deserializer, ethereum_types_serialize::ExpectedLen::Between(0, &mut bytes))?;
		// 		Ok(bytes[0..wrote].into())
		// 	}
		// }
	}
}

impl_serde!(H64, 8);
impl_serde!(H128, 16);
impl_serde!(H160, 20);
impl_serde!(H256, 32);
//impl_serde!(U256, 32);
impl_serde!(H512, 64);
impl_serde!(H520, 65);
impl_serde!(Bloom, 256);

pub type Address = H160;
pub type U256 = H256;
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
