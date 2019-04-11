/// Variable length integer
#[derive(Debug, Clone)]
pub struct VarInt(Vec<u8>);

impl From<VarInt> for Vec<u8> {
    fn from(v: VarInt) -> Vec<u8> {
        v.0
    }
}

impl From<VarInt> for Option<u64> {
    fn from(v: VarInt) -> Option<u64> {
        let v = v.0;
        if v.is_empty() {
            return None;
        }
        match v[0] {
            n @ 0x00..=0xfc => Some(u64::from(n)),
            0xfd if v.len() >= 3 => {
                let mut n = [0; 2];
                n.copy_from_slice(&v[1..3]);
                Some(u64::from(u16::from_le_bytes(n)))
            },
            0xfe if v.len() >= 5 => {
                let mut n = [0; 4];
                n.copy_from_slice(&v[1..5]);
                Some(u64::from(u32::from_le_bytes(n)))
            },
            0xff if v.len() >= 9 => {
                let mut n = [0; 8];
                n.copy_from_slice(&v[1..9]);
                Some(u64::from_le_bytes(n))
            },
            _ => None,
        }
    }
}

impl From<u64> for VarInt {
    fn from(num: u64) -> VarInt {
        match num {
            n @ 0x00..=0xfc => {
                VarInt([n as u8; 1].to_vec())
            },
            n @ 0xfd..=0xffff => {
                let mut v = [0; 3];
                v[0] = 0xfdu8;
                v[1..].copy_from_slice(&(n as u16).to_le_bytes());
                VarInt(v.to_vec())
            },
            n @ 0x10000..=0xffff_ffff => {
                let mut v = [0; 5];
                v[0] = 0xfeu8;
                v[1..].copy_from_slice(&(n as u32).to_le_bytes());
                VarInt(v.to_vec())
            },
            n => {
                let mut v = [0; 9];
                v[0] = 0xffu8;
                v[1..].copy_from_slice(&(n as u64).to_le_bytes());
                VarInt(v.to_vec())
            }
        } 
    }
}

impl VarInt {
    /// Convert into `Vec<u8>`
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }

    /// Convert into `Option<u64>`
    pub fn into_u64(self) -> Option<u64> {
        self.into()
    }

    /// Get data length
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return `true` if data length is zero
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convert `&[u8]` to `Option<VarInt>`
    pub fn from_slice(v: &[u8]) -> Option<VarInt> {
        if v.is_empty() {
            return None;
        }
        match v[0] {
            0x00..=0xfc          => Some(VarInt(v[0..1].to_vec())),
            0xfd if v.len() >= 3 => Some(VarInt(v[0..3].to_vec())),
            0xfe if v.len() >= 5 => Some(VarInt(v[0..5].to_vec())),
            0xff if v.len() >= 9 => Some(VarInt(v[0..9].to_vec())),
            _ => None,
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u64() {
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

            let vi: Option<u64> = VarInt::from_slice(*v).unwrap().into();
            assert_eq!(vi.unwrap(), *n);
        }
    }
}
