//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

mod errors;
mod identifier;
mod providers;
mod traits;

pub use errors::{Error, Result};
pub use identifier::Identifier;
pub use providers::*;
pub use traits::{ContentReader, ContentWriter};
