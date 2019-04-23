use std::convert::{TryFrom, TryInto};

use super::error::{Result, Error};

/// Variable length integer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VarInt(u64);

impl From<VarInt> for Vec<u8> {
    fn from(v: VarInt) -> Vec<u8> {
        match v.0 {
            n @ 0x00..=0xfc => {
                [n as u8; 1].to_vec()
            },
            n @ 0xfd..=0xffff => {
                let mut v = [0; 3];
                v[0] = 0xfdu8;
                v[1..].copy_from_slice(&(n as u16).to_le_bytes());
                v.to_vec()
            },
            n @ 0x10000..=0xffff_ffff => {
                let mut v = [0; 5];
                v[0] = 0xfeu8;
                v[1..].copy_from_slice(&(n as u32).to_le_bytes());
                v.to_vec()
            },
            n => {
                let mut v = [0; 9];
                v[0] = 0xffu8;
                v[1..].copy_from_slice(&(n as u64).to_le_bytes());
                v.to_vec()
            }
        }
    }
}

impl From<VarInt> for u64 {
    fn from(v: VarInt) -> u64 {
        v.0
    }
}

impl From<u64> for VarInt {
    fn from(num: u64) -> VarInt {
        VarInt(num)
    }
}

macro_rules! from_le_bytes {
    ($t: ident, $v: expr) => {
        Ok(
            $t::from_le_bytes(
                $v.try_into()
                    .or(Err(Error::TryFromVarIntError))?
            ).into()
        )
    };
}

// TODO: use AsRef<[u8]>
impl TryFrom<&[u8]> for VarInt {
    type Error = Error;

    fn try_from(v: &[u8]) -> Result<VarInt> {
        if v.is_empty() {
            return Err(Error::TryFromVarIntError);
        }
        let num = match v[0] {
            0x00..=0xfc          => Ok(v[0].into()),
            0xfd if v.len() >= 3 => from_le_bytes!(u16, &v[1..3]),
            0xfe if v.len() >= 5 => from_le_bytes!(u32, &v[1..5]),
            0xff if v.len() >= 9 => from_le_bytes!(u64, &v[1..9]),
            _ => Err(Error::TryFromVarIntError),
        }?;

        Ok(VarInt(num))
    }
}

#[allow(clippy::len_without_is_empty)]
impl VarInt {
    /// Return serialized length
    pub fn len(self) -> usize {
        match self.0 {
            0x00..=0xfc => 1,
            0xfd..=0xffff => 3,
            0x10000..=0xffff_ffff => 5,
            _ => 9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u64() -> Result<()> {
        let set: &[(u64, &[u8])] = &[
            (0x00, &[0x00]),
            (0xfc, &[0xfc]),
            (0xfd,        &[0xfd, 0xfd, 0x00]),
            (0xffff,      &[0xfd, 0xff, 0xff]),
            (0x10000,     &[0xfe, 0x00, 0x00, 0x01, 0x00]),
            (0xffff_ffff, &[0xfe, 0xff, 0xff, 0xff, 0xff]),
            (0x0001_0000_0000,      &[0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]),
            (0xffff_ffff_ffff_ffff, &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
        ];

        for (n, v) in set {
            let vi: Vec<u8> = VarInt::from(*n).into();
            assert_eq!(vi, v.to_vec());

            let vi: u64 = VarInt::try_from(*v)?.into();
            assert_eq!(vi, *n);
        }

        Ok(())
    }

    #[cfg(feature = "serde")]
    use serde_json;

    #[cfg(feature = "serde")]
    #[test]
    fn serde() -> std::result::Result<(), serde_json::error::Error> {
        let vi = VarInt::from(10);

        let serialized = serde_json::to_string(&vi)?;
        assert_eq!(serialized, "10");

        let deserialized: VarInt = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized, vi);

        Ok(())
    }
}
