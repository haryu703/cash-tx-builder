use std::result;

use failure::Fail;
use bch_addr;
use hex;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid index: {}", 0)]
    InvalidIndex(usize),
    #[fail(display = "Invalid lenght data: {}", 0)]
    InvalidLengthData(usize),
    #[fail(display = "bch_addr error: {}", 0)]
    BchAddrError(bch_addr::Error),
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
