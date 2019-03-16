mod input;
mod output;
mod var_int;

use input::Input;
use output::Output;
use var_int::VarInt;
use super::error::{Error, Result};
use bch_addr;
use super::script::{Script, address_to_script, null_data_script, encode};
use super::hash;
use sha2::{Sha256, Digest};
use super::bit_util::BitUtil;
use super::uint256::uint256;

pub mod sig_hash {
    pub const ALL: u32 = 0x01;
    pub const NONE: u32 = 0x02;
    pub const SINGLE: u32 = 0x03;
    pub const FORKID: u32 = 0x40;
    pub const ANYONECANPAY: u32 = 0x80;
}

pub struct TxBuilder<'a> {
    version: u32,
    inputs: Vec<Input>,
    prev_outputs: Vec<Output>,
    outputs: Vec<Output>,
    lock_time: u32,
    fork_id: u32,
    address_converter: &'a bch_addr::Converter,
}

impl<'a> From<&TxBuilder<'a>> for Vec<u8> {
    fn from(tb: &TxBuilder) -> Vec<u8> {
        [
            tb.version.to_le_bytes().to_vec(),
            VarInt::from(tb.inputs.len()).into(),
            tb.inputs.iter().flat_map(|p| p.to_vec()).collect(),
            VarInt::from(tb.outputs.len()).into(),
            tb.outputs.iter().flat_map(|p| p.to_vec()).collect(),
            tb.lock_time.to_le_bytes().to_vec(),
        ].concat()
    }
}

// impl From<Vec<u8>> for TxBuilder {
//     fn from(v: Vec<u8>) -> TxBuilder {}
// }

impl<'a> TxBuilder<'a> {
    pub fn new(address_converter: &'a bch_addr::Converter) -> TxBuilder<'a> {
        TxBuilder {
            version: 2,
            inputs: vec![],
            prev_outputs: vec![],
            outputs: vec![],
            lock_time: 0,
            fork_id: 0,
            address_converter,
        }
    }

    pub fn set_version(&mut self, v: u32) {
        self.version = v;
    }

    pub fn set_fork_id(&mut self, id: u32) {
        self.fork_id = id;
    }

    pub fn add_input(&mut self, txid: &str, index: u32, value: u64, script: &[u8], sequence_no: Option<u32>) -> Result<()> {
        let txid = uint256::try_from(txid)?;
        self.inputs.push(Input::new(&txid.into(), index, sequence_no));
        self.prev_outputs.push(Output::new(value, script));

        Ok(())
    }

    pub fn set_script_sig(&mut self, index: usize, script: &[u8]) -> Result<()> {
        let input = self.inputs.get_mut(index).ok_or_else(|| Error::InvalidIndex(index))?;
        input.set_script(script);
        Ok(())
    }

    pub fn add_address_output(&mut self, value: u64, address: &str) -> Result<()> {
        let script = address_to_script(address, self.address_converter)?;
        self.add_output(value, &script);
        Ok(())
    }

    pub fn add_null_data_output(&mut self, value: u64, data: &[u8]) -> Result<()> {
        let script = null_data_script(data)?;
        self.add_output(value, &script);
        Ok(())
    }

    pub fn add_output(&mut self, value: u64, script: &[u8]) {
        self.outputs.push(Output::new(value, script));
    }

    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(self)
    }

    pub fn witness_v0_hash(&self, hash_type: u32, index: u32) -> Result<Vec<u8>> {
        let hash_prev_outs = if !hash_type.is_set(sig_hash::ANYONECANPAY) {
            let hasher = self.inputs.iter().fold(Sha256::new(), |hasher, i| {
                hasher.chain(i.prev_txid).chain(i.prev_index.to_le_bytes())
            });
            hash::hash256(hasher)
        } else {
            vec![0; 32]
        };

        let hash_sequence = if !hash_type.is_set(sig_hash::ANYONECANPAY) && 
                               (hash_type & 0x1f) != sig_hash::SINGLE &&
                               (hash_type & 0x1f) != sig_hash::NONE {
            let hasher = self.inputs.iter().fold(Sha256::new(), |hasher, i| {
                hasher.chain(i.sequence_no.to_le_bytes())
            });
            hash::hash256(hasher)
        } else {
            vec![0; 32]
        };

        let hash_outputs = if (hash_type & 0x1f) != sig_hash::SINGLE &&
                              (hash_type & 0x1f) != sig_hash::NONE {
            let hasher = self.outputs.iter().fold(Sha256::new(), |hasher, o| {
                hasher.chain(o.to_vec())
            });
            hash::hash256(hasher)
        } else if (hash_type & 0x1f) == sig_hash::SINGLE &&
                  index < self.outputs.len() as u32 {
            let hasher = Sha256::new().chain(self.outputs[index as usize].to_vec());
            hash::hash256(hasher)
        } else {
            vec![0; 32]
        };

        let prev_output = self.prev_outputs.get(index as usize).ok_or_else(|| Error::InvalidIndex(index as usize))?;
        let input = self.inputs.get(index as usize).ok_or_else(|| Error::InvalidIndex(index as usize))?;

        let hasher = Sha256::new()
            .chain(self.version.to_le_bytes())
            .chain(hash_prev_outs)
            .chain(hash_sequence)
            .chain(input.prev_txid)
            .chain(input.prev_index.to_le_bytes())
            .chain(encode(&[Script::Data(&prev_output.script)])?)
            .chain(prev_output.value.to_le_bytes())
            .chain(input.sequence_no.to_le_bytes())
            .chain(hash_outputs)
            .chain(self.lock_time.to_le_bytes())
            .chain(((self.fork_id << 8) | hash_type).to_le_bytes());

        Ok(hash::hash256(hasher))
    }

    pub fn txid(&self) -> String {
        let hash = hash::hash256(Sha256::new().chain(self.to_vec()));
        uint256::from(hash).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::script::p2pkh;

    #[test]
    fn get_digest() {
        let converter = bch_addr::Converter::new();
        let mut txb = TxBuilder::new(&converter);
        let prev_txid = "427cfc8a960e6a33552c19bcfcbe9d59207248856fb8806ba9c7043421e1ee4c";
        let prev_index = 1;
        let prev_script = address_to_script("qq6zfutryz9rkem05rkpwq60pu5sxg4z5c330k4w75", &converter).unwrap();
        let prev_value = 100_000;

        txb.add_input(prev_txid, prev_index, prev_value, &prev_script, None).unwrap();
        txb.add_address_output(11000, "qqntvyp35r7l8julzldgh8qlc49x8rpkjyh4nz5ty3").unwrap();
        txb.add_address_output(88757, "qqny0aeaayxca8d4khmh68xp44d0aqwk3sk3zpzs70").unwrap();

        let script_sig = p2pkh::script_sig(
            &hex!("0366be8427eddf9341141e5bb10486e41b1f3b33101ab3d5e816c37f30f2ddb036"),
            &hex!("304402202dacf747f6ddc911b755938a07232cfa34057f7a336f72346c438c04f4d5dbc502206a7915ce8569ab5832dae89275bdc13f2467a69684643704f1a9a38b34d55b3041")
        ).unwrap();
        txb.set_script_sig(0, &script_sig).unwrap();

        let hash_type = sig_hash::ALL | sig_hash::FORKID;
        let sighash = txb.witness_v0_hash(hash_type, 0).unwrap();

        let txid = txb.txid();

        assert_eq!(sighash, hex!("2b492e7c4c8a3d670fd7fe324a87e3c55df1802c9a100f4006f8fff7c0913dd4"));
        assert_eq!(txid, "ec225c44df97f7573583c17f5b3fa55cc7bf4cc6b916ee88fd7cd3284e0dfcda");
    }
}
