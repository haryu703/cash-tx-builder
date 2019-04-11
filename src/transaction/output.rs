use super::super::var_int::VarInt;

#[derive(Debug)]
pub struct Output {
    pub value: u64,
    pub script: Vec<u8>,
}

impl From<&Output> for Vec<u8> {
    fn from(o: &Output) -> Vec<u8> {
        [
            &o.value.to_le_bytes()[..],
            &VarInt::from(o.script.len() as u64).into_vec(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_test() {
        let value = 10000;
        let script = hex!("76a91492fc13573caf1bd38bd65738428406f4af80793a88ac");

        let output = Output::new(value, &script);

        assert_eq!(output.value, value);
        assert_eq!(output.script, script);
        assert_eq!(output.to_vec(), hex!("10270000000000001976a91492fc13573caf1bd38bd65738428406f4af80793a88ac").to_vec());
    }
}
