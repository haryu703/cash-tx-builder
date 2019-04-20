mod error;
/// Transaction structures
pub mod transaction;
mod uint256;
mod var_int;

pub use error::Error as TypeError;
pub use var_int::*;
pub use uint256::uint256 as u256;
