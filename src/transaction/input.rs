use super::super::var_int::VarInt;

#[derive(Debug)]
pub struct Input {
    pub prev_txid: [u8; 32],
    pub prev_index: u32,
    pub script: Vec<u8>,
    pub sequence_no: u32,
}

impl From<&Input> for Vec<u8> {
    fn from(i: &Input) -> Vec<u8> {
        [
            &i.prev_txid[..],
            &i.prev_index.to_le_bytes(),
            &VarInt::from(i.script.len() as u64).into_vec(),
            &i.script,
            &i.sequence_no.to_le_bytes()[..],
        ].concat()
    }
}

impl Input {
    pub fn new(txid: &[u8; 32], index: u32, sequence_no: Option<u32>) -> Input {
        Input {
            prev_txid: *txid,
            prev_index: index,
            script: vec![],
            sequence_no: sequence_no.unwrap_or(0xffff_ffff),
        }
    }

    pub fn set_script(&mut self, script: &[u8]) {
        self.script = script.to_vec();
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::uint256::uint256;

    #[test]
    fn construct_test() {
        let txid = uint256::try_from("695538649751ffdb1a28c4c8bf9dca9afe5b65a3dbaea25770105aa2154b9a33").unwrap().into();
        let index = 1;
        let script = hex!("47304402204bdde4960e3733c64b8debc7c2ce609699e418de91e055594a7fd53f07e618b90220066f02e1f9a3e26e76ff4220de3b2b17dab63684c1fb9ef567ed2056ba3a96d44121030a7decd850db8d31c819bd34a0f9934f9c51e1f78718f59c886a3c8389c0d1de");

        let mut input = Input::new(&txid, index, None);
        input.set_script(&script);

        assert_eq!(input.prev_index, index);
        assert_eq!(input.prev_txid, txid);
        assert_eq!(input.script, script.to_vec());
        assert_eq!(input.sequence_no, 0xffff_ffff);

        assert_eq!(input.to_vec(), hex!("339a4b15a25a107057a2aedba3655bfe9aca9dbfc8c4281adbff519764385569010000006a47304402204bdde4960e3733c64b8debc7c2ce609699e418de91e055594a7fd53f07e618b90220066f02e1f9a3e26e76ff4220de3b2b17dab63684c1fb9ef567ed2056ba3a96d44121030a7decd850db8d31c819bd34a0f9934f9c51e1f78718f59c886a3c8389c0d1deffffffff").to_vec());
    }
}
