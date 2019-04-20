//! P2PKH utility

use super::super::script::{encode, Script};
use super::super::opcode::OpCode::*;
use super::super::error::{Result};

/// Build `scriptPubKey` from hashed `public key`
/// # Arguments
/// * `hash` - Hashed `public key`
/// # Returns
/// * `scriptPubKey`
/// # Example
/// ```
/// # #[macro_use] extern crate hex_literal;
/// # use cash_tx_builder::script::p2pkh::script_pub_key;
/// let hash = hex!("023a723c9e8b8297d84f6ab7dc08784c36b0729a");
/// let script_pub_key = script_pub_key(&hash)?;
/// assert_eq!(script_pub_key, hex!("76a914023a723c9e8b8297d84f6ab7dc08784c36b0729a88ac"));
/// # Ok::<(), cash_tx_builder::Error>(())
/// ```
pub fn script_pub_key(hash: &[u8]) -> Result<Vec<u8>> {
    encode(&[
        Script::OpCode(OP_DUP),
        Script::OpCode(OP_HASH160),
        Script::Data(hash),
        Script::OpCode(OP_EQUALVERIFY),
        Script::OpCode(OP_CHECKSIG),
    ])
}

/// Build `scriptSig` from `public key` and `signature`
/// # Arguments
/// * `pubkey` - `public key`
/// * `sig` - transaction's `signature`
/// # Returns
/// * `scriptSig`
/// # Example
/// ```
/// # #[macro_use] extern crate hex_literal;
/// # use cash_tx_builder::script::p2pkh::script_sig;
/// let pubkey = hex!("0366be8427eddf9341141e5bb10486e41b1f3b33101ab3d5e816c37f30f2ddb036");
/// let sig = hex!("304402202dacf747f6ddc911b755938a07232cfa34057f7a336f72346c438c04f4d5dbc502206a7915ce8569ab5832dae89275bdc13f2467a69684643704f1a9a38b34d55b3041");
/// let script_sig = script_sig(&pubkey, &sig)?;
/// assert_eq!(script_sig, hex!("47304402202dacf747f6ddc911b755938a07232cfa34057f7a336f72346c438c04f4d5dbc502206a7915ce8569ab5832dae89275bdc13f2467a69684643704f1a9a38b34d55b3041210366be8427eddf9341141e5bb10486e41b1f3b33101ab3d5e816c37f30f2ddb036").to_vec());
/// # Ok::<(), cash_tx_builder::Error>(())
/// ```
pub fn script_sig(pubkey: &[u8], sig: &[u8]) -> Result<Vec<u8>> {
    encode(&[
        Script::Data(sig),
        Script::Data(pubkey),
    ])
}
