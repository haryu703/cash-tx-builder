#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(unused)]
#![warn(nonstandard_style)]
#![warn(rust_2018_idioms)]

//! transaction builder for bitcoin cash
//! # Example
//! ```
//! #[macro_use] extern crate hex_literal;
//! use bch_addr::{AddressType, Converter};
//! use cash_tx_builder::{TxBuilder, sig_hash};
//! use cash_tx_builder::script::{address_to_script, p2pkh};
//! 
//! let converter = Converter::new();
//! let parser = |address: &str| {
//!     let parsed = converter.parse(address).ok();
//!     match parsed {
//!         Some((_, _, address_type, hash)) => {
//!             Some((hash, address_type == AddressType::P2PKH))
//!         }
//!         None => None
//!     }
//! };
//! let mut txb = TxBuilder::new(&parser);
//! let prev_txid = "427cfc8a960e6a33552c19bcfcbe9d59207248856fb8806ba9c7043421e1ee4c";
//! let prev_index = 1;
//! let prev_script = address_to_script("qq6zfutryz9rkem05rkpwq60pu5sxg4z5c330k4w75", &parser).unwrap();
//! let prev_value = 100_000;
//! 
//! txb.add_input(prev_txid, prev_index, prev_value, &prev_script, None).unwrap();
//! 
//! txb.add_address_output(11000, "qqntvyp35r7l8julzldgh8qlc49x8rpkjyh4nz5ty3").unwrap();
//! txb.add_address_output(88757, "qqny0aeaayxca8d4khmh68xp44d0aqwk3sk3zpzs70").unwrap();
//! 
//! let script_sig = p2pkh::script_sig(
//!     &hex!("0366be8427eddf9341141e5bb10486e41b1f3b33101ab3d5e816c37f30f2ddb036"),
//!     &hex!("304402202dacf747f6ddc911b755938a07232cfa34057f7a336f72346c438c04f4d5dbc502206a7915ce8569ab5832dae89275bdc13f2467a69684643704f1a9a38b34d55b3041")
//! ).unwrap();
//! txb.set_script_sig(0, &script_sig).unwrap();
//! 
//! let hash_type = sig_hash::ALL | sig_hash::FORKID;
//! let sighash = txb.witness_v0_hash(hash_type, 0).unwrap();
//! let txid = txb.txid();
//! 
//! assert_eq!(sighash, hex!("2b492e7c4c8a3d670fd7fe324a87e3c55df1802c9a100f4006f8fff7c0913dd4"));
//! assert_eq!(txid, "ec225c44df97f7573583c17f5b3fa55cc7bf4cc6b916ee88fd7cd3284e0dfcda");
//! ```

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
