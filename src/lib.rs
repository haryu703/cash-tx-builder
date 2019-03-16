mod tx_builder;
mod error;
mod opcode;
pub mod script;
mod hash;
mod bit_util;
mod uint256;

#[cfg(test)]
#[macro_use]
extern crate hex_literal;

pub use error::{Error, Result};
pub use opcode::OpCode;
pub use tx_builder::{TxBuilder, sig_hash};
