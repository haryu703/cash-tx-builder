use crate::types::u256;

/// Outpoint
#[allow(missing_docs)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OutPoint {
    pub txid: u256,
    pub n: u32,
}

impl From<&OutPoint> for Vec<u8> {
    fn from(op: &OutPoint) -> Vec<u8> {
        let mut ret = [0; 36];

        ret[0..32].copy_from_slice(op.txid.as_ref());
        ret[32..36].copy_from_slice(&op.n.to_le_bytes());

        ret.to_vec()
    }
}
