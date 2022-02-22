//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

mod errors;
mod identifier;
mod providers;
mod traits;

pub use errors::{Error, Result};
pub use identifier::{HashAlgorithm, Identifier};
pub use providers::*;
pub use traits::{
    ContentAddressProvider, ContentAddressReader, ContentAddressWriter, ContentAsyncRead,
    ContentAsyncWrite, ContentProvider, ContentReader, ContentWriter,
};
