pub mod p2pkh;
pub mod p2sh;

use bch_addr::{Converter, AddressType};

use super::opcode::OpCode;
use OpCode::*;
use super::error::{Error, Result};

pub enum Script<'a> {
    OpCode(OpCode),
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
        l @ 0x11...0x4b => {
            v.push(l as u8);
            v.extend(data);
        },
        l @ 0x4c...0xff => {
            v.push(OP_PUSHDATA1 as u8);
            v.extend(&(l as u8).to_le_bytes());
            v.extend(data);
        },
        l @ 0x100...0xffff => {
            v.push(OP_PUSHDATA2 as u8);
            v.extend(&(l as u16).to_le_bytes());
            v.extend(data);
        },
        l @ 0x10000...0xffff_ffff => {
            v.push(OP_PUSHDATA4 as u8);
            v.extend(&(l as u32).to_le_bytes());
            v.extend(data);
        },
        e => return Err(Error::InvalidLengthData(e)),
    };
    Ok(())
}

pub fn encode(scripts: &[Script]) -> Result<Vec<u8>> {
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
