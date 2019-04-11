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
