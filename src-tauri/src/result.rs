use crate::error::Error;

/// The result of every command.
pub type Result<T> = std::result::Result<T, Error>;
