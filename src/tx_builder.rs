use std::collections::HashMap;
use std::str::FromStr;

use super::error::{Error, Result};
use super::script::{Script, address_to_script, null_data_script, encode};
use super::hash;
use sha2::{Sha256, Digest};
use super::bit_util::BitUtil;
use super::types::u256;
use super::types::transaction::Transaction;
use super::types::transaction::input::Input;
use super::types::transaction::output::Output;

/// sighash type
pub mod sig_hash {
    #![allow(missing_docs)]
    pub const ALL: u32 = 0x01;
    pub const NONE: u32 = 0x02;
    pub const SINGLE: u32 = 0x03;
    pub const FORKID: u32 = 0x40;
    pub const ANYONECANPAY: u32 = 0x80;
}

/// Transaction builder
#[derive(Debug)]
pub struct TxBuilder<F> 
        where F: Fn(&str) -> Option<(Vec<u8>, bool)> {
    tx: Transaction,
    prev_outputs: HashMap<usize, Output>,
    fork_id: u32,
    address_parser: F,
}

impl<F: Fn(&str) -> Option<(Vec<u8>, bool)>> TxBuilder<F> {
    /// Construct new transaction builder
    /// # Arguments
    /// * `address_parser` - address parser closure
    ///     ## Arguments
    ///     * address
    ///     ## Returns
    ///     * hashed `public key` or hashed `redeem script`
    ///     * `true` if address is P2PKH, `false` if address is P2SH
    /// 
    ///     or `None`
    pub fn new(address_parser: F) -> TxBuilder<F> {
        TxBuilder {
            tx: Transaction::new(),
            prev_outputs: HashMap::new(),
            fork_id: 0,
            address_parser,
        }
    }

    /// Construct transaction builder from `Transaction`
    /// # Arguments
    /// * `tx` - transaction
    /// * `address_parser` - address parser closure
    ///     ## Arguments
    ///     * address
    ///     ## Returns
    ///     * hashed `public key` or hashed `redeem script`
    ///     * `true` if address is P2PKH, `false` if address is P2SH
    /// 
    ///     or `None`
    /// # Example
    /// ```
    /// # #[macro_use] extern crate hex_literal;
    /// # use std::convert::TryFrom;
    /// # use bch_addr::{AddressType, Converter};
    /// # use cash_tx_builder::TxBuilder;
    /// # use cash_tx_builder::types::transaction::Transaction;
    /// # let converter = Converter::new();
    /// # let parser = |address: &str| {
    /// #     let parsed = converter.parse(address).ok();
    /// #     match parsed {
    /// #         Some((_, _, address_type, hash)) => {
    /// #             Some((hash, address_type == AddressType::P2PKH))
    /// #         }
    /// #         None => None
    /// #     }
    /// # };
    /// let hex = hex!("0100000001339a4b15a25a107057a2aedba3655bfe9aca9dbfc8c4281adbff519764385569010000006a47304402204bdde4960e3733c64b8debc7c2ce609699e418de91e055594a7fd53f07e618b90220066f02e1f9a3e26e76ff4220de3b2b17dab63684c1fb9ef567ed2056ba3a96d44121030a7decd850db8d31c819bd34a0f9934f9c51e1f78718f59c886a3c8389c0d1deffffffff02d7f52d01000000001976a914214ffcd3e7668da243cc4006759f6fe5f3c60bfe88ac10270000000000001976a91492fc13573caf1bd38bd65738428406f4af80793a88ac00000000");
    /// let txid = "7bdc016701e4c5d7ec34e99954ec3921140728d2c58b1da3cf6aa34c760d8a47";
    /// let tx = Transaction::try_from(&hex[..])?;
    /// let txb = TxBuilder::from_tx(&tx, parser)?;
    /// assert_eq!(txb.txid(), txid);
    /// # Ok::<(), cash_tx_builder::Error>(())
    /// ```
    pub fn from_tx(tx: &Transaction, address_parser: F) -> Result<TxBuilder<F>> {
        Ok(TxBuilder {
            tx: tx.clone(),
            prev_outputs: HashMap::new(),
            fork_id: 0,
            address_parser,
        })
    }

    /// Set transaction version (default: 2)
    /// # Arguments
    /// `v` - version
    /// # Example
    /// ```
    /// # use bch_addr::{AddressType, Converter};
    /// # use cash_tx_builder::TxBuilder;
    /// # let converter = Converter::new();
    /// # let parser = |address: &str| {
    /// #     let parsed = converter.parse(address).ok();
    /// #     match parsed {
    /// #         Some((_, _, address_type, hash)) => {
    /// #             Some((hash, address_type == AddressType::P2PKH))
    /// #         }
    /// #         None => None
    /// #     }
    /// # };
    /// # let mut txb = TxBuilder::new(&parser);
    /// txb.set_version(1);
    /// assert_eq!(txb.to_vec()[0..4], (0x01 as u32).to_le_bytes());
    /// ```
    pub fn set_version(&mut self, v: u32) {
        self.tx.version = v;
    }

    /// Set fork id (default: 0)
    /// # Arguments
    /// * `id` - forkid
    /// # Example
    /// ```
    /// # #[macro_use] extern crate hex_literal;
    /// # use bch_addr::{AddressType, Converter};
    /// # use cash_tx_builder::{TxBuilder, sig_hash};
    /// # use cash_tx_builder::script::{address_to_script, p2pkh};
    /// # let converter = Converter::new();
    /// # let parser = |address: &str| {
    /// #     let parsed = converter.parse(address).ok();
    /// #     match parsed {
    /// #         Some((_, _, address_type, hash)) => {
    /// #             Some((hash, address_type == AddressType::P2PKH))
    /// #         }
    /// #         None => None
    /// #     }
    /// # };
    /// # let mut txb = TxBuilder::new(&parser);
    /// # let prev_txid = "427cfc8a960e6a33552c19bcfcbe9d59207248856fb8806ba9c7043421e1ee4c";
    /// # let prev_index = 1;
    /// # let prev_script = address_to_script("qq6zfutryz9rkem05rkpwq60pu5sxg4z5c330k4w75", &parser).unwrap();
    /// # let prev_value = 100_000;
    /// # txb.add_input(prev_txid, prev_index, Some(prev_value), Some(&prev_script), None).unwrap();
    /// # txb.add_address_output(11000, "qqntvyp35r7l8julzldgh8qlc49x8rpkjyh4nz5ty3").unwrap();
    /// # txb.add_address_output(88757, "qqny0aeaayxca8d4khmh68xp44d0aqwk3sk3zpzs70").unwrap();
    /// # let script_sig = p2pkh::script_sig(
    /// #     &hex!("0366be8427eddf9341141e5bb10486e41b1f3b33101ab3d5e816c37f30f2ddb036"),
    /// #     &hex!("304402202dacf747f6ddc911b755938a07232cfa34057f7a336f72346c438c04f4d5dbc502206a7915ce8569ab5832dae89275bdc13f2467a69684643704f1a9a38b34d55b3041")
    /// # ).unwrap();
    /// # txb.set_script_sig(0, &script_sig).unwrap();
    /// let hash_type = sig_hash::ALL | sig_hash::FORKID;
    /// txb.set_fork_id(1);
    /// let sighash = txb.witness_v0_hash(hash_type, 0, None, None).unwrap();
    /// assert_eq!(sighash, hex!("e99f87c70a16dfa390ea0ddd0748709a8548b7b97c62f91754d74264c687db62"));
    /// ```
    pub fn set_fork_id(&mut self, id: u32) {
        self.fork_id = id;
    }

    /// Add input
    /// # Arguments
    /// * `txid` - previous transaction hash
    /// * `index` - previous txout-index
    /// * `value` - (option) previous value
    /// * `script` - (option) previous `scriptPubKey`
    /// * `sequence_no`- (option) sequence number
    pub fn add_input(&mut self, txid: &str, index: u32, value: Option<u64>, script: Option<&[u8]>, sequence_no: Option<u32>) -> Result<()> {
        let txid = u256::from_str(txid)?;
        self.tx.inputs.push(Input::new(&txid.into(), index, sequence_no));
        if value.is_some() && script.is_some() {
            self.prev_outputs.insert(
                self.tx.inputs.len() - 1,
                Output::new(value.unwrap(), script.unwrap())
            );
        }

        Ok(())
    }

    /// Set `scriptSig`
    /// # Arguments
    /// * `index` - previous txout-index
    /// * `script` - `scriptSig`
    pub fn set_script_sig(&mut self, index: usize, script: &[u8]) -> Result<()> {
        let input = self.tx.inputs.get_mut(index).ok_or_else(|| Error::InvalidIndex(index))?;
        input.script = script.to_vec();
        Ok(())
    }

    /// Add output by bitcoin address
    /// # Arguments
    /// * `value` - satoshi
    /// * `address` - bitcoin address
    pub fn add_address_output(&mut self, value: u64, address: &str) -> Result<()> {
        let script = address_to_script(address, &self.address_parser)?;
        self.add_output(value, &script);
        Ok(())
    }

    /// Add output by null data
    /// # Arguments
    /// * `data` - extra data
    /// # Example
    /// ```
    /// # use bch_addr::{AddressType, Converter};
    /// # use cash_tx_builder::TxBuilder;
    /// # let converter = Converter::new();
    /// # let parser = |address: &str| {
    /// #     let parsed = converter.parse(address).ok();
    /// #     match parsed {
    /// #         Some((_, _, address_type, hash)) => {
    /// #             Some((hash, address_type == AddressType::P2PKH))
    /// #         }
    /// #         None => None
    /// #     }
    /// # };
    /// # let mut txb = TxBuilder::new(&parser);
    /// txb.add_null_data_output(b"hoge");
    /// assert_eq!(&txb.to_vec()[17..21], b"hoge");
    /// ```
    pub fn add_null_data_output(&mut self, data: &[u8]) -> Result<()> {
        let script = null_data_script(data)?;
        self.add_output(0, &script);
        Ok(())
    }

    /// Add output by null data
    /// # Arguments
    /// * `value` - satoshi
    /// * `script` - `scriptPubKey`
    /// # Example
    /// ```
    /// # #[macro_use] extern crate hex_literal;
    /// # use bch_addr::{AddressType, Converter};
    /// # use cash_tx_builder::TxBuilder;
    /// # let converter = Converter::new();
    /// # let parser = |address: &str| {
    /// #     let parsed = converter.parse(address).ok();
    /// #     match parsed {
    /// #         Some((_, _, address_type, hash)) => {
    /// #             Some((hash, address_type == AddressType::P2PKH))
    /// #         }
    /// #         None => None
    /// #     }
    /// # };
    /// # let mut txb = TxBuilder::new(&parser);
    /// let script = hex!("76a91432b57f34861bcbe33a701be9ac3a50288fbc0a3d88ac");
    /// txb.add_output(1000, &script);
    /// assert_eq!(&txb.to_vec()[15..40], script);
    /// ```
    pub fn add_output(&mut self, value: u64, script: &[u8]) {
        self.tx.outputs.push(Output::new(value, script));
    }

    /// Convert to `Vec<u8>`
    /// # Returns
    /// * serialized transaction
    pub fn to_vec(&self) -> Vec<u8> {
        Vec::from(&self.tx)
    }

    /// Get digest according to bip143  
    /// [spec](https://github.com/Bitcoin-ABC/bitcoin-abc/blob/master/doc/abc/replay-protected-sighash.md)
    /// # Arguments
    /// * `hash_type` - sighash type
    /// * `index` - input index
    /// * `prev_value` - (option) previous value
    /// * `prev_script` - (option) previous script
    pub fn witness_v0_hash(&self, hash_type: u32, index: u32, prev_value: Option<u64>, prev_script: Option<&[u8]>) -> Result<Vec<u8>> {
        let hash_prev_outs = if !hash_type.is_set(sig_hash::ANYONECANPAY) {
            let hasher = self.tx.inputs.iter().fold(Sha256::new(), |hasher, i| {
                hasher.chain(i.outpoint.txid).chain(i.outpoint.n.to_le_bytes())
            });
            hash::hash256(hasher)
        } else {
            vec![0; 32]
        };

        let hash_sequence = if !hash_type.is_set(sig_hash::ANYONECANPAY) && 
                               (hash_type & 0x1f) != sig_hash::SINGLE &&
                               (hash_type & 0x1f) != sig_hash::NONE {
            let hasher = self.tx.inputs.iter().fold(Sha256::new(), |hasher, i| {
                hasher.chain(i.sequence_no.to_le_bytes())
            });
            hash::hash256(hasher)
        } else {
            vec![0; 32]
        };

        let hash_outputs = if (hash_type & 0x1f) != sig_hash::SINGLE &&
                              (hash_type & 0x1f) != sig_hash::NONE {
            let hasher = self.tx.outputs.iter().fold(Sha256::new(), |hasher, o| {
                hasher.chain(o.to_vec())
            });
            hash::hash256(hasher)
        } else if (hash_type & 0x1f) == sig_hash::SINGLE &&
                  index < self.tx.outputs.len() as u32 {
            let hasher = Sha256::new().chain(self.tx.outputs[index as usize].to_vec());
            hash::hash256(hasher)
        } else {
            vec![0; 32]
        };

        let (prev_value, prev_script) = if prev_value.is_some() && prev_script.is_some() {
            (prev_value.unwrap(), prev_script.unwrap())
        } else if let Some(o) = self.prev_outputs.get(&(index as usize)) {
            (o.value, &o.script[..])
        } else {
            return Err(Error::InvalidIndex(index as usize));
        };

        let input = self.tx.inputs.get(index as usize).ok_or_else(|| Error::InvalidIndex(index as usize))?;

        let hasher = Sha256::new()
            .chain(self.tx.version.to_le_bytes())
            .chain(hash_prev_outs)
            .chain(hash_sequence)
            .chain(input.outpoint.txid)
            .chain(input.outpoint.n.to_le_bytes())
            .chain(encode(&[Script::Data(&prev_script)])?)
            .chain(prev_value.to_le_bytes())
            .chain(input.sequence_no.to_le_bytes())
            .chain(hash_outputs)
            .chain(self.tx.lock_time.to_le_bytes())
            .chain(((self.fork_id << 8) | hash_type).to_le_bytes());

        Ok(hash::hash256(hasher))
    }

    /// Get txid
    /// # Returns
    /// * txid
    pub fn txid(&self) -> String {
        let hash = hash::hash256(Sha256::new().chain(self.to_vec()));
        u256::from(&hash[..]).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::script::p2pkh;
    use bch_addr::{AddressType, Converter};

    #[test]
    fn get_digest() {
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

        let mut txb = TxBuilder::new(&parser);
        let prev_txid = "427cfc8a960e6a33552c19bcfcbe9d59207248856fb8806ba9c7043421e1ee4c";
        let prev_index = 1;
        let prev_script = address_to_script("qq6zfutryz9rkem05rkpwq60pu5sxg4z5c330k4w75", &parser).unwrap();
        let prev_value = 100_000;

        txb.add_input(prev_txid, prev_index, Some(prev_value), Some(&prev_script), None).unwrap();
        txb.add_address_output(11000, "qqntvyp35r7l8julzldgh8qlc49x8rpkjyh4nz5ty3").unwrap();
        txb.add_address_output(88757, "qqny0aeaayxca8d4khmh68xp44d0aqwk3sk3zpzs70").unwrap();

        let script_sig = p2pkh::script_sig(
            &hex!("0366be8427eddf9341141e5bb10486e41b1f3b33101ab3d5e816c37f30f2ddb036"),
            &hex!("304402202dacf747f6ddc911b755938a07232cfa34057f7a336f72346c438c04f4d5dbc502206a7915ce8569ab5832dae89275bdc13f2467a69684643704f1a9a38b34d55b3041")
        ).unwrap();
        txb.set_script_sig(0, &script_sig).unwrap();

        let hash_type = sig_hash::ALL | sig_hash::FORKID;
        let sighash = txb.witness_v0_hash(hash_type, 0, None, None).unwrap();

        let txid = txb.txid();

        assert_eq!(sighash, hex!("2b492e7c4c8a3d670fd7fe324a87e3c55df1802c9a100f4006f8fff7c0913dd4"));
        assert_eq!(txid, "ec225c44df97f7573583c17f5b3fa55cc7bf4cc6b916ee88fd7cd3284e0dfcda");
    }
}
