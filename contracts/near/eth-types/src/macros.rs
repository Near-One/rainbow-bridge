#[cfg(feature = "eth2")]
#[macro_export]
macro_rules! arr_wrapper_impl_tree_hash_and_borsh {
    ($name: ident, $len: expr) => {
        impl tree_hash::TreeHash for $name {
            fn tree_hash_type() -> tree_hash::TreeHashType {
                tree_hash::TreeHashType::Vector
            }

            fn tree_hash_packed_encoding(&self) -> Vec<u8> {
                unreachable!("Vector should never be packed.")
            }

            fn tree_hash_packing_factor() -> usize {
                unreachable!("Vector should never be packed.")
            }

            fn tree_hash_root(&self) -> tree_hash::Hash256 {
                let values_per_chunk = tree_hash::BYTES_PER_CHUNK;
                let minimum_chunk_count = ($len + values_per_chunk - 1) / values_per_chunk;
                tree_hash::merkle_root(&self.0, minimum_chunk_count)
            }
        }

        impl BorshSerialize for $name {
            #[inline]
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                writer.write_all(&self.0)?;
                Ok(())
            }
        }

        impl BorshDeserialize for $name {
            #[inline]
            fn deserialize(buf: &mut &[u8]) -> Result<Self, Error> {
                if buf.len() < $len {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Unexpected length of input",
                    ));
                }
                let mut data = [0u8; $len];
                data.copy_from_slice(&buf[..$len]);
                *buf = &buf[$len..];
                Ok($name(data.into()))
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

        #[cfg(not(target_arch = "wasm32"))]
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
            where
                D: Deserializer<'de>,
            {
                let mut s = <String as Deserialize>::deserialize(deserializer)?;
                if s.starts_with("0x") {
                    s = s[2..].to_string();
                }
                let result =
                    Vec::from_hex(&s).map_err(|err| serde::de::Error::custom(err.to_string()))?;
                Ok(result.into())
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&format!("0x{}", hex::encode(self.0)))
            }
        }
    };
}

#[cfg(feature = "eth2")]
#[macro_export]
macro_rules! vec_wrapper_impl_tree_hash {
    ($name: ident) => {
        impl tree_hash::TreeHash for $name {
            fn tree_hash_type() -> tree_hash::TreeHashType {
                tree_hash::TreeHashType::Vector
            }

            fn tree_hash_packed_encoding(&self) -> Vec<u8> {
                unreachable!("Vector should never be packed.")
            }

            fn tree_hash_packing_factor() -> usize {
                unreachable!("Vector should never be packed.")
            }

            fn tree_hash_root(&self) -> tree_hash::Hash256 {
                let mut hasher = MerkleHasher::with_leaves(self.0.len());

                for item in &self.0 {
                    hasher
                        .write(item.tree_hash_root().as_bytes())
                        .expect("ssz_types vec should not contain more elements than max");
                }

                hasher
                    .finish()
                    .expect("ssz_types vec should not have a remaining buffer")
            }
        }
    };
}

#[macro_export]
macro_rules! arr_ethereum_types_wrapper_impl_borsh_serde_ssz {
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
                        "Unexpected length of input",
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

        #[cfg(feature = "eth2")]
        impl ssz::Decode for $name {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                $len
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
                let len = bytes.len();
                let expected = <Self as ssz::Decode>::ssz_fixed_len();

                if len != expected {
                    Err(ssz::DecodeError::InvalidByteLength { len, expected })
                } else {
                    Ok(bytes.into())
                }
            }
        }

        #[cfg(feature = "eth2")]
        impl ssz::Encode for $name {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                $len
            }

            fn ssz_bytes_len(&self) -> usize {
                $len
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                buf.extend_from_slice(self.0.as_bytes());
            }
        }
    };
}
