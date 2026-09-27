use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::Macros;

/// The user's daily figures. There is at most one set.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Targets {
    pub macros: Macros,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
