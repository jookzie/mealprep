use std::fmt::Display;

use mealprep_core::{Entity, Error, Result};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

/// Wraps a failure of the database, or of what it holds, which the user cannot act on.
pub(crate) fn internal(source: impl Display) -> Error {
    Error::Internal {
        reason: source.to_string(),
    }
}

pub(crate) fn not_found(entity: Entity, id: impl Display) -> Error {
    Error::NotFound {
        entity,
        id: id.to_string(),
    }
}

/// Returns the current time as it is stored: RFC 3339 text in UTC.
pub(crate) fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("internal error: the current UTC time always formats as RFC 3339")
}

pub(crate) fn parse_timestamp(value: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(internal)
}

pub(crate) fn parse_id(value: &str) -> Result<Uuid> {
    Uuid::parse_str(value).map_err(internal)
}

pub(crate) fn parse_optional_id(value: Option<&str>) -> Result<Option<Uuid>> {
    value.map(parse_id).transpose()
}

/// Encodes the given ids as a JSON array, which queries unpack with `json_each`.
pub(crate) fn id_list<'id>(ids: impl IntoIterator<Item = &'id Uuid>) -> String {
    let ids: Vec<_> = ids.into_iter().map(Uuid::to_string).collect();
    serde_json::to_string(&ids).expect("internal error: a list of strings always encodes as JSON")
}

/// Encodes the given optional id the way `coalesce(column, '')` reads it.
pub(crate) fn scope_key(id: Option<Uuid>) -> String {
    id.map(|id| id.to_string()).unwrap_or_default()
}
