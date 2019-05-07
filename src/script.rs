//! utility of bitcoin script

pub mod p2pkh;
pub mod p2sh;

use num_traits::FromPrimitive;
use std::convert::TryInto;
use super::opcode::OpCode;
use OpCode::*;
use super::error::{Error, Result};

/// Element to build bitcoin script
#[derive(Debug, PartialEq)]
pub enum Script<'a> {
    /// op code
    OpCode(OpCode),
    /// data
    Data(&'a [u8]),
}

const DATA_OPCODE: [OpCode; 17] = [
    OP_0, OP_1, OP_2, OP_3, OP_4, OP_5, OP_6, OP_7, OP_8,
    OP_9, OP_10, OP_11, OP_12, OP_13, OP_14, OP_15, OP_16,
];

fn push_data(data: &[u8], v: &mut Vec<u8>) -> Result<()> {
    match data.len() {
        0 => {
            v.push(OP_0 as u8);
        },
        1 if data[0] <= 16 => {
            v.push(DATA_OPCODE[data[0] as usize] as u8);
        },
        1 if data[0] == 0x81 => {
            v.push(OP_1NEGATE as u8);
        },
        l @ 0x00..=0x4b => {
            v.push(l as u8);
            v.extend(data);
        },
        l @ 0x4c..=0xff => {
            v.push(OP_PUSHDATA1 as u8);
            v.extend(&(l as u8).to_le_bytes());
            v.extend(data);
        },
        l @ 0x100..=0xffff => {
            v.push(OP_PUSHDATA2 as u8);
            v.extend(&(l as u16).to_le_bytes());
            v.extend(data);
        },
        l @ 0x10000..=0xffff_ffff => {
            v.push(OP_PUSHDATA4 as u8);
            v.extend(&(l as u32).to_le_bytes());
            v.extend(data);
        },
        e => return Err(Error::InvalidLengthData(e)),
    };
    Ok(())
}

/// Build raw script from scripts
/// # Arguments
/// * `scripts` - array of `Script`
/// # Returns
/// * raw script
/// # Example
/// ```
/// # #[macro_use] extern crate hex_literal;
/// # use cash_tx_builder::script::{Script, encode};
/// # use cash_tx_builder::OpCode::*;
/// let scripts = [
///     Script::OpCode(OP_DUP),
///     Script::OpCode(OP_HASH160),
///     Script::Data(&hex!("023a723c9e8b8297d84f6ab7dc08784c36b0729a")),
///     Script::OpCode(OP_EQUALVERIFY),
///     Script::OpCode(OP_CHECKSIG),
/// ];
/// let encoded = encode(&scripts)?;
/// assert_eq!(encoded, hex!("76a914023a723c9e8b8297d84f6ab7dc08784c36b0729a88ac"));
/// # Ok::<(), cash_tx_builder::Error>(())
/// ```
pub fn encode(scripts: &[Script<'_>]) -> Result<Vec<u8>> {
    scripts.iter().try_fold(Vec::new(), |mut v, script| {
        match script {
            Script::OpCode(op) => {
                v.push(*op as u8);
            },
            Script::Data(data) => {
                push_data(data, &mut v)?;
            },
        };
        Ok(v)
    })
}

fn get_opcode(v: &[u8]) -> Option<(Script<'_>, &[u8])> {
    let op = v.get(0)?;
    let v = v.get(1..)?;

    if *op <= 0x4b {
        let len = *op as usize;
        return Some((Script::Data(v.get(..len)?), v.get(len..)?));
    }

    match OpCode::from_u8(*op) {
        Some(OP_PUSHDATA1) => {
            let len = *v.get(0)? as usize;
            let v = v.get(1..)?;
            Some((Script::Data(v.get(..len)?), v.get(len..)?))
        },
        Some(OP_PUSHDATA2) => {
            let len = u16::from_le_bytes(v.get(..2)?.try_into().ok()?) as usize;
            let v = v.get(2..)?;
            Some((Script::Data(v.get(..len)?), v.get(len..)?))
        },
        Some(OP_PUSHDATA4) => {
            let len = u32::from_le_bytes(v.get(..4)?.try_into().ok()?) as usize;
            let v = v.get(4..)?;
            Some((Script::Data(v.get(..len)?), v.get(len..)?))
        },
        Some(op) => Some((Script::OpCode(op), v)),
        None => None,
    }
}

/// Decode raw script to array of `Script`
/// # Arguments
/// * v - raw script
/// # Returns
/// * array of `Script`
/// # Example
/// ```
/// # #[macro_use] extern crate hex_literal;
/// # use cash_tx_builder::script::{Script, decode};
/// # use cash_tx_builder::OpCode::*;
/// let hex = hex!("76a914023a723c9e8b8297d84f6ab7dc08784c36b0729a88ac");
/// let scripts = [
///     Script::OpCode(OP_DUP),
///     Script::OpCode(OP_HASH160),
///     Script::Data(&hex!("023a723c9e8b8297d84f6ab7dc08784c36b0729a")),
///     Script::OpCode(OP_EQUALVERIFY),
///     Script::OpCode(OP_CHECKSIG),
/// ];
/// let decoded = decode(&hex)?;
/// assert_eq!(decoded, scripts);
/// # Ok::<(), cash_tx_builder::Error>(())
/// ```
pub fn decode(v: &[u8]) -> Result<Vec<Script<'_>>> {
    let mut scripts = Vec::new();
    let mut cur = v;
    while !cur.is_empty() {
        if let Some((script, n)) = get_opcode(cur) {
            scripts.push(script);
            cur = n;
        } else {
            return Err(Error::InvalidOpCode(cur[0]));
        }
    }

    Ok(scripts)
}

/// Convert address to `scriptPubKey`
/// # Arguments
/// * `address` - bitcoin address
/// * `parser` - address parser
/// # Returns
/// * `scriptPubKey`
/// # Example
/// ```
/// # #[macro_use] extern crate hex_literal;
/// # use bch_addr::{AddressType, Converter};
/// # use cash_tx_builder::script::address_to_script;
/// let converter = Converter::new();
/// let parser = |address: &str| {
///     let parsed = converter.parse(address).ok();
///     match parsed {
///         Some((_, _, address_type, hash)) => {
///             Some((hash, address_type == AddressType::P2PKH))
///         }
///         None => None
///     }
/// };
/// 
/// let p2pkh = "bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk";
/// let p2sh = "bitcoincash:pph5kuz78czq00e3t85ugpgd7xmer5kr7crv8a2z4t";
/// 
/// let p2pkh_script = address_to_script(p2pkh, &parser)?;
/// let p2sh_script = address_to_script(p2sh, &parser)?;
/// 
/// assert_eq!(p2pkh_script, hex!("76a9146f4b705e3e0407bf3159e9c4050df1b791d2c3f688ac"));
/// assert_eq!(p2sh_script, hex!("a9146f4b705e3e0407bf3159e9c4050df1b791d2c3f687"));
/// # Ok::<(), cash_tx_builder::Error>(())
/// ```
pub fn address_to_script<F>(address: &str, parser: &F) -> Result<Vec<u8>>
    where F: Fn(&str) -> Option<(Vec<u8>, bool)> {
    let (hash, is_pkh) = parser(address).ok_or_else(|| Error::InvalidAddress(address.to_string()))?;

    if is_pkh {
        p2pkh::script_pub_key(&hash)
    } else {
        p2sh::script_pub_key(&hash)
    }
}

/// Build `scriptPubKey` from `null data`
/// # Arguments
/// * `data` - null data
/// # Returns
/// * `scriptPubKey`
/// # Example
/// ```
/// # #[macro_use] extern crate hex_literal;
/// # use cash_tx_builder::script::null_data_script;
/// let data = hex!("1234567890");
/// let script_pub_key = null_data_script(&data)?;
/// assert_eq!(script_pub_key, hex!("6a051234567890"));
/// # Ok::<(), cash_tx_builder::Error>(())
/// ```
pub fn null_data_script(data: &[u8]) -> Result<Vec<u8>> {
    encode(&[
        Script::OpCode(OP_RETURN),
        Script::Data(data),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use bch_addr::{AddressType, Converter};

    #[test]
    fn get_p2pkh() -> Result<()> {
        let converter = Converter::new();
        let parser = |address: &str| {
            let parsed = converter.parse(address).ok();
            match parsed {
                Some((_, _, address_type, hash)) => {
                    Some((hash, address_type == AddressType::P2PKH))
                }
                None => None
            }
        };
        let script = address_to_script("qq6zfutryz9rkem05rkpwq60pu5sxg4z5c330k4w75", &parser)?;
        assert_eq!(script, hex!("76a9143424f163208a3b676fa0ec17034f0f290322a2a688ac"));

        Ok(())
    }

    #[test]
    fn decode_test() -> Result<()> {
        let hex = hex!("76a914023a723c9e8b8297d84f6ab7dc08784c36b0729a88ac");
        let scripts = [
            Script::OpCode(OP_DUP),
            Script::OpCode(OP_HASH160),
            Script::Data(&hex!("023a723c9e8b8297d84f6ab7dc08784c36b0729a")),
            Script::OpCode(OP_EQUALVERIFY),
            Script::OpCode(OP_CHECKSIG),
        ];

        let decoded = decode(&hex)?;
        assert_eq!(decoded, scripts);

        Ok(())
    }
}
