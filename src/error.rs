use std::result;

use failure::Fail;
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

    /// Invalid bitcoin address.
    /// # Arguments
    /// * address
    #[fail(display = "Invalid address: {}", 0)]
    InvalidAddress(String),

    /// Transaction parse error
    /// # Arguments
    /// * error index
    /// * raw transaction
    #[fail(display = "Transaction parse error: at {}, {:?}", 0, 1)]
    TxParseError(usize, Vec<u8>),

    /// hex library's error.
    /// # Arguments
    /// * error
    #[fail(display = "hex error: {}", 0)]
    HexError(hex::FromHexError),
}

impl From<hex::FromHexError> for Error {
    fn from(err: hex::FromHexError) -> Error {
        Error::HexError(err)
    }
}
