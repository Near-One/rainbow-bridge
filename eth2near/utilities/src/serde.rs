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
    let s = extract_hex_string(deserializer)?;

    N::from_str_radix(&s, 16)
        .map_err(|_| de::Error::custom(format!("Invalid hex string: {}", s)))
}

impl<'de, const N: usize> Deserialize<'de> for FixedBytes<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        let s = extract_hex_string(deserializer)?;
        let padded = format!("{:0>size$}", s, size = N * 2);

        let bytes: [u8; N] = hex::decode(&padded)
            .map_err(|_| de::Error::custom(format!("Invalid hex string: {}", padded)))?
            .try_into()
            .map_err(|_| de::Error::custom(format!("Hex string of invalid length: {}", padded)))?;

        Ok(FixedBytes::<N>(bytes))
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        let s = extract_hex_string(deserializer)?;

        let vec: Vec<u8> = hex::decode(&s)
            .map_err(|_| de::Error::custom(format!("Invalid hex string: {}", s)))?;

        Ok(Bytes(vec))
    }
}

impl<'de> Deserialize<'de> for Byte {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        Ok(Byte(hex_to_int(deserializer)?))
    }
}

fn extract_hex_string<'de, D>(deserializer: D) -> Result<String, D::Error>
    where D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if let Some(stripped) = s.strip_prefix("0x") {
        Ok(match stripped.len() % 2 { 
            0 => stripped.to_owned(),
            _ => "0".to_owned() + &stripped,
        })
    } else {
        Err(de::Error::custom(format!("Hex string must start with 0x: {}", s)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_fixed() {
        let s = r#""0x1234""#;
        let bytes: FixedBytes<2> = serde_json::from_str(s).unwrap();
        assert_eq!(bytes.0, [0x12, 0x34]);

        let s = r#""0x23456""#;
        let bytes: FixedBytes<3> = serde_json::from_str(s).unwrap();
        assert_eq!(bytes.0, [0x02, 0x34, 0x56]);

        let s = r#""0x23456""#;
        let bytes: FixedBytes<8> = serde_json::from_str(s).unwrap();
        assert_eq!(bytes.0, [0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x34, 0x56]);
    }

    #[test]
    fn deserialize_bytes() {
        let s = r#""0x1234""#;
        let bytes: Bytes = serde_json::from_str(s).unwrap();
        assert_eq!(bytes.0, vec![0x12, 0x34]);

        let s = r#""0x21234""#;
        let bytes: Bytes = serde_json::from_str(s).unwrap();
        assert_eq!(bytes.0, vec![0x02, 0x12, 0x34]);

        let s = r#""0x0""#;
        let bytes: Bytes = serde_json::from_str(s).unwrap();
        assert_eq!(bytes.0, vec![0x00]);
    }

    #[test]
    fn deserialize_byte() {
        let s = r#""0x11""#;
        let byte: Byte = serde_json::from_str(s).unwrap();
        assert_eq!(byte.0, 0x11);

        let s = r#""0x1""#;
        let byte: Byte = serde_json::from_str(s).unwrap();
        assert_eq!(byte.0, 0x01);

        let s = r#""0x0""#;
        let byte: Byte = serde_json::from_str(s).unwrap();
        assert_eq!(byte.0, 0x00);
    }

    #[test]
    fn deserialize_num() {
        let s = r#""0x1234""#;        
        let deserializer = &mut serde_json::Deserializer::from_str(s);
        let num: u16 = hex_to_int(deserializer).unwrap();
        assert_eq!(num, 0x1234);

        let s = r#""0x234""#;        
        let deserializer = &mut serde_json::Deserializer::from_str(s);
        let num: u16 = hex_to_int(deserializer).unwrap();
        assert_eq!(num, 0x0234);

        let s = r#""0x234""#;        
        let deserializer = &mut serde_json::Deserializer::from_str(s);
        let num: u32 = hex_to_int(deserializer).unwrap();
        assert_eq!(num, 0x00000234);
    }
}

