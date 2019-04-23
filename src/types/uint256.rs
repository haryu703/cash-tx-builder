use std::str::FromStr;
use hex;

use super::error::{Error, Result};

#[cfg(feature = "serde")]
use serde::{Serializer, Serialize, Deserializer, Deserialize, de};

/// 256 bit unsigned value
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct uint256(pub [u8; 32]);

// TODO: use AsRef<[u8]>
impl From<&[u8]> for uint256 {
    fn from(v: &[u8]) -> uint256 {
        let mut array = [0; 32];

        let src = if v.len() > 32 {
            &v[0..31]
        } else {
            v
        };
        let dest = if v.len() < 32 {
            &mut array[0..v.len()]
        } else {
            &mut array
        };
        dest.copy_from_slice(src);

        uint256(array)
    }
}

impl From<uint256> for [u8; 32] {
    fn from(v: uint256) -> [u8; 32] {
        v.0
    }
}

impl From<uint256> for String {
    fn from(v: uint256) -> String {
        v.0.iter().rev().map(|v| format!("{:02x}", v)).collect()
    }
}

impl AsRef<[u8]> for uint256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl FromStr for uint256 {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let v = hex::decode(s)?.into_iter().rev().collect::<Vec<u8>>();

        Ok(v[..].into())
    }
}

#[cfg(feature = "serde")]
impl Serialize for uint256 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&String::from(*self))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for uint256 {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;

        Ok(uint256::from_str(&s)
        .or_else(|_| {
            Err(de::Error::invalid_value(
                de::Unexpected::Str(&s),
                &"hex string",
            ))
        })?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert() -> Result<()> {
        let v_str = uint256::from_str("ec225c44df97f7573583c17f5b3fa55cc7bf4cc6b916ee88fd7cd3284e0dfcda")?;

        let mut arr = [0; 32];
        arr.copy_from_slice(v_str.as_ref());

        let v_arr = uint256::from(arr.as_ref());
        assert_eq!(v_str, v_arr);

        Ok(())
    }

    #[cfg(feature = "serde")]
    use serde_json;

    #[cfg(feature = "serde")]
    mod error {
        use crate::TypeError;

        #[derive(Debug)]
        pub enum SerdeTestError {
            TypeError(TypeError),
            SerdeError(serde_json::error::Error),
        }

        impl From<TypeError> for SerdeTestError {
            fn from(err: TypeError) -> SerdeTestError {
                SerdeTestError::TypeError(err)
            }
        }

        impl From<serde_json::error::Error> for SerdeTestError {
            fn from(err: serde_json::error::Error) -> SerdeTestError {
                SerdeTestError::SerdeError(err)
            }
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde() -> std::result::Result<(), error::SerdeTestError> {
        let v = uint256::from_str("ec225c44df97f7573583c17f5b3fa55cc7bf4cc6b916ee88fd7cd3284e0dfcda")?;

        let serialized = serde_json::to_string(&v)?;
        assert_eq!(serialized, "\"ec225c44df97f7573583c17f5b3fa55cc7bf4cc6b916ee88fd7cd3284e0dfcda\"");

        let deserialized: uint256 = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized, v);

        Ok(())
    }
}
