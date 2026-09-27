use std::{io, path::PathBuf};

use mealprep_core::Error as CoreError;
use serde::Serialize;

/// A failed command, as the frontend receives it.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(rename_all = "camelCase")]
#[error("{message}")]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

/// What went wrong, so the frontend can react without parsing the message.
#[derive(Copy, Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorKind {
    CatalogueUnavailable,
    Conflict,
    Internal,
    Invalid,
    NotFound,
}

impl From<CoreError> for Error {
    fn from(error: CoreError) -> Self {
        let kind = match &error {
            CoreError::NotFound { .. } => ErrorKind::NotFound,
            CoreError::Invalid { .. } => ErrorKind::Invalid,
            CoreError::Conflict { .. } => ErrorKind::Conflict,
            CoreError::CatalogueUnavailable { .. } => ErrorKind::CatalogueUnavailable,
            CoreError::Internal { .. } => ErrorKind::Internal,
        };
        let message = error.to_string();
        Self { kind, message }
    }
}

/// A failure to start the application.
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("cannot locate the application data directory: {reason}")]
    LocateDataDirectory { reason: String },

    #[error("cannot create {path}: {source}")]
    CreateDataDirectory { path: PathBuf, source: io::Error },

    #[error(transparent)]
    Mealprep(#[from] CoreError),
}
