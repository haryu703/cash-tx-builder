#[derive(Debug, Clone)]
pub struct VarInt(Vec<u8>);

impl From<VarInt> for Vec<u8> {
    fn from(v: VarInt) -> Vec<u8> {
        v.0
    }
}

impl From<usize> for VarInt {
    fn from(num: usize) -> VarInt {
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
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_usize() {
        let set: &[(usize, &[u8])] = &[
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
        }
    }
}
