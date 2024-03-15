use num_traits::Num;
use serde::{de, Deserialize, Deserializer};
use crate::primitives::{Byte, Bytes, FixedBytes};

pub fn hex_to_int_opt<'de, D, N>(deserializer: D) -> Result<Option<N>, D::Error>
    where D: Deserializer<'de>,
          N: Num,
{
    Ok(Some(hex_to_int(deserializer)?))
}

pub fn hex_to_int<'de, D, N>(deserializer: D) -> Result<N, D::Error>
    where D: Deserializer<'de>,
          N: Num,
{
    match String::deserialize(deserializer)? {
        s if s.starts_with("0x") => {
            N::from_str_radix(&s[2..], 16)
                .map_err(|_| de::Error::custom(format!("Invalid hex string: {}", s)))
        }
        s => Err(de::Error::custom(format!("Hex string must start with 0x: {}", s))),
    }
}

impl<'de, const N: usize> Deserialize<'de> for FixedBytes<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)? {
            s if s.starts_with("0x") => {
                let s_normalized = normalize_hex_string(&s);

                let bytes: [u8; N] = hex::decode(&s_normalized)
                    .map_err(|_| de::Error::custom(format!("Invalid hex string: {}", s)))?
                    .try_into()
                    .map_err(|_| de::Error::custom(format!("Invalid hex string: {}", s)))?;

                Ok(FixedBytes::<N>(bytes))
            }
            s => Err(de::Error::custom(format!("Hex string must start with 0x: {}", s))),
        }
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)? {
            s if s.starts_with("0x") => {
                let s_normalized = normalize_hex_string(&s);

                let vec: Vec<u8> = hex::decode(&s_normalized)
                    .map_err(|_| de::Error::custom(format!("Invalid hex string: {}", s)))?;

                Ok(Bytes(vec))
            }
            s => Err(de::Error::custom(format!("Hex string must start with 0x: {}", s))),
        }
    }
}

impl<'de> Deserialize<'de> for Byte {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)? {
            s if s.starts_with("0x") => {
                let s_normalized = normalize_hex_string(&s);

                let bytes: Vec<u8> = hex::decode(&s_normalized)
                    .map_err(|_| de::Error::custom(format!("Invalid hex string: {}", s)))?;

                Ok(Byte(bytes[0]))
            }
            s => Err(de::Error::custom(format!("Hex string must start with 0x: {}", s))),
        }
    }
}

fn normalize_hex_string(s: &str) -> String {
    match s.len() % 2 { 
        0 => s[2..].to_owned(),
        _ => "0".to_owned() + &s[2..],
    }
}
