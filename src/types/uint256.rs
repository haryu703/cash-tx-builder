use std::str::FromStr;
use hex;

use super::error::{Error, Result};

/// 256 bit unsigned value
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
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
