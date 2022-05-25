use thiserror::Error;

/// An error type.
#[derive(Error, Debug)]
pub enum Error {}

/// A result type.
pub type Result<T> = std::result::Result<T, Error>;
