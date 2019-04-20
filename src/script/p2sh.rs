//! P2SH utility

use super::super::script::{encode, Script};
use super::super::opcode::OpCode::*;
use super::super::error::{Result};

/// Build `scriptPubKey` from hashed `redeem script`
/// # Arguments
/// * `hash` - Hashed `redeem script`
/// # Returns
/// * `scriptPubKey`
/// # Example
/// ```
/// # #[macro_use] extern crate hex_literal;
/// # use cash_tx_builder::script::p2sh::script_pub_key;
/// let hash = hex!("023a723c9e8b8297d84f6ab7dc08784c36b0729a");
/// let script_pub_key = script_pub_key(&hash)?;
/// assert_eq!(script_pub_key, hex!("a914023a723c9e8b8297d84f6ab7dc08784c36b0729a87"));
/// # Ok::<(), cash_tx_builder::Error>(())
/// ```
pub fn script_pub_key(hash: &[u8]) -> Result<Vec<u8>> {
    encode(&[
        Script::OpCode(OP_HASH160),
        Script::Data(hash),
        Script::OpCode(OP_EQUAL),
    ])
}
