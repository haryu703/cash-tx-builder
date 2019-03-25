use std::result;

use failure::Fail;
use bch_addr;
use hex;

/// Alias of `Result` used by cash_tx_builder.
pub type Result<T> = result::Result<T, Error>;

/// Errors
#[derive(Debug, Fail)]
pub enum Error {
    /// Invalid input/output index.
    /// # Arguments
    /// * index
    #[fail(display = "Invalid index: {}", 0)]
    InvalidIndex(usize),

    /// Invalid length script.
    /// # Arguments
    /// * data length
    #[fail(display = "Invalid lenght data: {}", 0)]
    InvalidLengthData(usize),

    /// bch_addr library's error.
    /// # Arguments
    /// * error
    #[fail(display = "bch_addr error: {}", 0)]
    BchAddrError(bch_addr::Error),

    /// hex library's error.
    /// # Arguments
    /// * error
    #[fail(display = "hex error: {}", 0)]
    HexError(hex::FromHexError),
}

impl From<bch_addr::Error> for Error {
    fn from(err: bch_addr::Error) -> Error {
        Error::BchAddrError(err)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(err: hex::FromHexError) -> Error {
        Error::HexError(err)
    }
}
