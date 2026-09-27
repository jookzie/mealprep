//! Domain types, business rules and storage contracts of Mealprep.
//!
//! This crate knows nothing about where entities are persisted or where the product
//! catalogue lives: [`store`] declares what the rules need, and other crates provide it.

pub mod domain;
pub mod service;
pub mod store;

use std::fmt;

/// Every failure the business rules report.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot find {entity} {id}")]
    NotFound { entity: Entity, id: String },

    #[error("invalid {entity}: {reason}")]
    Invalid { entity: Entity, reason: String },

    #[error("cannot name {entity} {name:?}: another one in the same scope has that name")]
    Conflict { entity: Entity, name: String },

    #[error("cannot reach the product catalogue: {reason}")]
    CatalogueUnavailable { reason: String },

    #[error("internal error: {reason}")]
    Internal { reason: String },
}

/// The result of every fallible operation in Mealprep.
pub type Result<T> = std::result::Result<T, Error>;

/// The kind of entity an [`Error`] is about.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Entity {
    CalendarDay,
    CatalogueEntry,
    Category,
    DayPlan,
    Meal,
    Product,
    Targets,
    WeightEntry,
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::CalendarDay => "calendar day",
            Self::CatalogueEntry => "catalogue entry",
            Self::Category => "category",
            Self::DayPlan => "day plan",
            Self::Meal => "meal",
            Self::Product => "product",
            Self::Targets => "targets",
            Self::WeightEntry => "weight entry",
        };
        f.write_str(name)
    }
}
