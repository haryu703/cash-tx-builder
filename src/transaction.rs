pub mod input;
pub mod output;

use input::Input;
use output::Output;
use super::var_int::VarInt;
use super::error::{Error, Result};

/// Bitcoin Cash transaction format
#[derive(Default, Debug)]
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
    let vi = VarInt::from_slice(v)?;
    let size = vi.len();

    Some((vi.into_u64()?, &v[size..]))
}

impl From<&Transaction> for Vec<u8> {
    fn from(tx: &Transaction) -> Vec<u8> {
        [
            tx.version.to_le_bytes().to_vec(),
            VarInt::from(tx.inputs.len() as u64).into(),
            tx.inputs.iter().flat_map(|p| p.to_vec()).collect(),
            VarInt::from(tx.outputs.len() as u64).into(),
            tx.outputs.iter().flat_map(|p| p.to_vec()).collect(),
            tx.lock_time.to_le_bytes().to_vec(),
        ].concat()
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

    /// Construct `Transaction` from raw transaction
    /// # Arguments
    /// * `bytes` - raw transaction
    pub fn from_bytes(bytes: &[u8]) -> Result<Transaction> {
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
            input.set_script(script);
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
