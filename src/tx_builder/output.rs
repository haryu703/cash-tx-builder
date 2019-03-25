use super::var_int::VarInt;

#[derive(Debug)]
pub struct Output {
    pub value: u64,
    pub script: Vec<u8>,
}

impl From<&Output> for Vec<u8> {
    fn from(o: &Output) -> Vec<u8> {
        [
            &o.value.to_le_bytes()[..],
            &VarInt::from(o.script.len()).into_vec(),
            &o.script,
        ].concat()
    }
}

impl Output {
    pub fn new(value: u64, script: &[u8]) -> Output {
        Output {
            value,
            script: script.to_vec(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.into()
    }
}
