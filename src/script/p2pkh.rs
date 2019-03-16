use super::super::script::{encode, Script};
use super::super::opcode::OpCode::*;
use super::super::error::{Result};

pub fn script_pub_key(hash: &[u8]) -> Result<Vec<u8>> {
    encode(&[
        Script::OpCode(OP_DUP),
        Script::OpCode(OP_HASH160),
        Script::Data(hash),
        Script::OpCode(OP_EQUALVERIFY),
        Script::OpCode(OP_CHECKSIG),
    ])
}

pub fn script_sig(pubkey: &[u8], sig: &[u8]) -> Result<Vec<u8>> {
    encode(&[
        Script::Data(sig),
        Script::Data(pubkey),
    ])
}
