use crate::types::{VarInt, u256};
use super::OutPoint;

/// Transaction input
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    pub outpoint: OutPoint,
    pub script: Vec<u8>,
    pub sequence_no: u32,
}

impl From<&Input> for Vec<u8> {
    fn from(i: &Input) -> Vec<u8> {
        [
            &(&i.outpoint).into(),
            &Vec::from(VarInt::from(i.script.len() as u64)),
            &i.script,
            &i.sequence_no.to_le_bytes()[..],
        ].concat()
    }
}

impl Input {
    /// Construct `Input`
    /// # Arguments
    /// * txid - previous transaction hash
    /// * index - previous transaction output index
    /// * sequence_no - (option) sequence number
    pub fn new(txid: &[u8; 32], index: u32, sequence_no: Option<u32>) -> Input {
        Input {
            outpoint: OutPoint {txid: u256(*txid), n: index},
            script: vec![],
            sequence_no: sequence_no.unwrap_or(0xffff_ffff),
        }
    }

    /// Convert to `Vec<u8>`
    pub fn to_vec(&self) -> Vec<u8> {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::Result;
    use crate::u256;

    #[test]
    fn construct_test() -> Result<()> {
        let txid = u256::from_str("695538649751ffdb1a28c4c8bf9dca9afe5b65a3dbaea25770105aa2154b9a33")?.into();
        let index = 1;
        let script = hex!("47304402204bdde4960e3733c64b8debc7c2ce609699e418de91e055594a7fd53f07e618b90220066f02e1f9a3e26e76ff4220de3b2b17dab63684c1fb9ef567ed2056ba3a96d44121030a7decd850db8d31c819bd34a0f9934f9c51e1f78718f59c886a3c8389c0d1de");

        let mut input = Input::new(&txid, index, None);
        input.script = script.to_vec();

        assert_eq!(input.outpoint.n, index);
        assert_eq!(input.outpoint.txid.as_ref(), txid);
        assert_eq!(input.script, script.to_vec());
        assert_eq!(input.sequence_no, 0xffff_ffff);

        assert_eq!(input.to_vec(), hex!("339a4b15a25a107057a2aedba3655bfe9aca9dbfc8c4281adbff519764385569010000006a47304402204bdde4960e3733c64b8debc7c2ce609699e418de91e055594a7fd53f07e618b90220066f02e1f9a3e26e76ff4220de3b2b17dab63684c1fb9ef567ed2056ba3a96d44121030a7decd850db8d31c819bd34a0f9934f9c51e1f78718f59c886a3c8389c0d1deffffffff").to_vec());

        Ok(())
    }
}
