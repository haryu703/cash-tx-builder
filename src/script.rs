//! utility of bitcoin script

pub mod p2pkh;
pub mod p2sh;

use bch_addr::{Converter, AddressType};

use super::opcode::OpCode;
use OpCode::*;
use super::error::{Error, Result};

/// Element to build bitcoin script
#[derive(Debug)]
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
/// let encoded = encode(&scripts).unwrap();
/// assert_eq!(encoded, hex!("76a914023a723c9e8b8297d84f6ab7dc08784c36b0729a88ac"));
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

/// Convert address to `scriptPubKey`
/// # Arguments
/// * `address` - bitcoin address
/// * `converter` - address converter
/// # Returns
/// * `scriptPubKey`
/// # Example
/// ```
/// # #[macro_use] extern crate hex_literal;
/// # use bch_addr::Converter;
/// # use cash_tx_builder::script::address_to_script;
/// let converter = Converter::new();
/// 
/// let p2pkh = "bitcoincash:qph5kuz78czq00e3t85ugpgd7xmer5kr7c5f6jdpwk";
/// let p2sh = "bitcoincash:pph5kuz78czq00e3t85ugpgd7xmer5kr7crv8a2z4t";
/// 
/// let p2pkh_script = address_to_script(p2pkh, &converter).unwrap();
/// let p2sh_script = address_to_script(p2sh, &converter).unwrap();
/// 
/// assert_eq!(p2pkh_script, hex!("76a9146f4b705e3e0407bf3159e9c4050df1b791d2c3f688ac"));
/// assert_eq!(p2sh_script, hex!("a9146f4b705e3e0407bf3159e9c4050df1b791d2c3f687"));
/// ```
pub fn address_to_script(address: &str, converter: &Converter) -> Result<Vec<u8>>{
    let (_, _, addr_type, hash) = converter.parse(address)?;

    match addr_type {
        AddressType::P2PKH => {
            p2pkh::script_pub_key(&hash)
        },
        AddressType::P2SH => {
            p2sh::script_pub_key(&hash)
        },
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
/// let script_pub_key = null_data_script(&data).unwrap();
/// assert_eq!(script_pub_key, hex!("6a051234567890"));
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

    #[test]
    fn get_p2pkh() {
        let converter = Converter::new();
        let script = address_to_script("qq6zfutryz9rkem05rkpwq60pu5sxg4z5c330k4w75", &converter).unwrap();
        assert_eq!(script, hex!("76a9143424f163208a3b676fa0ec17034f0f290322a2a688ac"));
    }
}
