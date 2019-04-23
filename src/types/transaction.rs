/// Outpoint
pub mod outpoint;
/// Transaction input
pub mod input;
/// Transaction output
pub mod output;

use std::convert::TryFrom;
pub use outpoint::OutPoint;
pub use input::Input;
pub use output::Output;
use super::var_int::VarInt;
use super::error::{Error, Result};

/// Bitcoin Cash transaction format
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Transaction {
    /// version no
    pub version: u32,
    /// list of inputs
    pub inputs: Vec<Input>,
    /// list of outputs
    pub outputs: Vec<Output>,
    /// lock_time
    pub lock_time: u32,
}

fn read_bytes<T: Default + AsMut<[u8]>>(v: &[u8]) -> Option<(T, &[u8])> {
    let mut ret = T::default();
    let size = std::mem::size_of::<T>();
    if size > v.len() {
        return None;
    }
    ret.as_mut().copy_from_slice(&v[..size]);

    Some((ret, &v[size..]))
}

fn read_var_int(v: &[u8]) -> Option<(u64, &[u8])> {
    let vi = VarInt::try_from(v).ok()?;
    let size = vi.len();

    Some((vi.into(), &v[size..]))
}

impl From<&Transaction> for Vec<u8> {
    fn from(tx: &Transaction) -> Vec<u8> {
        [
            &tx.version.to_le_bytes()[..],
            &Vec::from(VarInt::from(tx.inputs.len() as u64)),
            &tx.inputs.iter().flat_map(Input::to_vec).collect::<Vec<u8>>()[..],
            &Vec::from(VarInt::from(tx.outputs.len() as u64)),
            &tx.outputs.iter().flat_map(Output::to_vec).collect::<Vec<u8>>()[..],
            &tx.lock_time.to_le_bytes(),
        ].concat()
    }
}

impl TryFrom<&[u8]> for Transaction {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Transaction> {
        let len = bytes.len();
        let mut tx = Transaction::new();

        let (version, read_pointer) = read_bytes(bytes)
                .ok_or_else(|| Error::TxParseError(0, bytes.to_vec()))?;
        let version = u32::from_le_bytes(version);
        tx.version = version;

        let (in_counter, mut read_pointer) = read_var_int(read_pointer)
                .ok_or_else(|| Error::TxParseError(len - read_pointer.len(), read_pointer.to_vec()))?;

        // parse input
        for _ in 0..in_counter {
            let (txid, p) = read_bytes(read_pointer)
                    .ok_or_else(|| Error::TxParseError(len - read_pointer.len(), read_pointer.to_vec()))?;

            let (index, p) = read_bytes(p)
                    .ok_or_else(|| Error::TxParseError(len - p.len(), p.to_vec()))?;
            let index = u32::from_le_bytes(index);

            let (script_len, p) = read_var_int(p)
                    .ok_or_else(|| Error::TxParseError(len - p.len(), p.to_vec()))?;

            if p.len() < script_len as usize {
                return Err(Error::TxParseError(len - p.len(), p.to_vec()));
            }
            let (script, p) = p.split_at(script_len as usize);

            let (sequence_no, p) = read_bytes(p)
                    .ok_or_else(|| Error::TxParseError(len - p.len(), p.to_vec()))?;
            let sequence_no = u32::from_le_bytes(sequence_no);

            let mut input = Input::new(&txid, index, Some(sequence_no));
            input.script = script.to_vec();
            tx.inputs.push(input);

            read_pointer = p;
        }

        let (out_counter, mut read_pointer) = read_var_int(read_pointer)
                .ok_or_else(|| Error::TxParseError(len - read_pointer.len(), read_pointer.to_vec()))?;
        
        // parse output
        for _ in 0..out_counter {
            let (value, p) = read_bytes(read_pointer)
                    .ok_or_else(|| Error::TxParseError(len - read_pointer.len(), read_pointer.to_vec()))?;
            let value = u64::from_le_bytes(value);

            let (script_len, p) = read_var_int(p)
                    .ok_or_else(|| Error::TxParseError(len - p.len(), p.to_vec()))?;
            
            if p.len() < script_len as usize {
                return Err(Error::TxParseError(len - p.len(), p.to_vec()));
            }
            let (script, p) = p.split_at(script_len as usize);

            let output = Output::new(value, script);
            tx.outputs.push(output);

            read_pointer = p;
        }

        let (lock_time, _) = read_bytes(read_pointer)
                .ok_or_else(|| Error::TxParseError(len - read_pointer.len(), read_pointer.to_vec()))?;
        let lock_time = u32::from_le_bytes(lock_time);

        tx.lock_time = lock_time;

        Ok(tx)
    }
}

impl Transaction {
    /// Construct new `Transaction`
    pub fn new() -> Transaction {
        Transaction {
            version: 2,
            inputs: vec![],
            outputs: vec![],
            lock_time: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use super::*;

    #[test]
    fn test_convert() -> Result<()> {
        let hex = hex!("0100000001339a4b15a25a107057a2aedba3655bfe9aca9dbfc8c4281adbff519764385569010000006a47304402204bdde4960e3733c64b8debc7c2ce609699e418de91e055594a7fd53f07e618b90220066f02e1f9a3e26e76ff4220de3b2b17dab63684c1fb9ef567ed2056ba3a96d44121030a7decd850db8d31c819bd34a0f9934f9c51e1f78718f59c886a3c8389c0d1deffffffff02d7f52d01000000001976a914214ffcd3e7668da243cc4006759f6fe5f3c60bfe88ac10270000000000001976a91492fc13573caf1bd38bd65738428406f4af80793a88ac00000000");

        let tx = Transaction::try_from(&hex[..])?;

        let tx_hex: Vec<u8> = (&tx).into();

        assert_eq!(tx_hex, hex.to_vec());

        Ok(())
    }
}
