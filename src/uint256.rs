use hex;
use super::error::Result;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Default)]
pub struct uint256(pub [u8; 32]);

impl<T: AsRef<[u8]>> From<T> for uint256 {
    fn from(v: T) -> uint256 {
        let v = v.as_ref();
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

impl uint256 {
    pub fn new() -> uint256 {
        uint256([0; 32])
    }

    pub fn try_from<T: AsRef<str>>(s: T) -> Result<uint256> {
        let s = s.as_ref();
        let v = hex::decode(s)?.into_iter().rev().collect::<Vec<u8>>();

        Ok(v.into())
    }
}
