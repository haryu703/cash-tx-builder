use std::result;

use failure::Fail;

/// Alias of `Result` used by cash_tx_builder.
pub type Result<T> = result::Result<T, Error>;

/// Errors
#[derive(Debug, Fail)]
pub enum Error {
    /// Transaction parse error
    /// # Arguments
    /// * error index
    /// * raw transaction
    #[fail(display = "Transaction parse error: at {}, {:?}", 0, 1)]
    TxParseError(usize, Vec<u8>),

    /// Convert error
    #[fail(display = "VarInt convert error")]
    TryFromVarIntError,

    /// hex library's error
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
