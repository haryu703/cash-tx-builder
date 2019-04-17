use std::result;

use failure::Fail;
use super::types;

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

    /// type error
    /// # Arguments
    /// * error
    #[fail(display = "type error: {}", 0)]
    TypeError(types::TypeError),
}

impl From<types::TypeError> for Error {
    fn from(err: types::TypeError) -> Error {
        Error::TypeError(err)
    }
}
