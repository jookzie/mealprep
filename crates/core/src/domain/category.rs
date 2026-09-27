use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{Entity, Error};

/// A name meals or day plans are grouped under, unique within its scope.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub scope: CategoryScope,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// What the user writes when creating a category.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryDraft {
    pub name: String,
    pub scope: CategoryScope,
}

/// What a category groups. Meals and day plans keep a list each, so one name can serve both.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CategoryScope {
    Meal,
    DayPlan,
}

impl CategoryScope {
    /// Returns the name this scope is stored and sent as.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Meal => "meal",
            Self::DayPlan => "day-plan",
        }
    }

    /// Names what categories of this scope group, for messages.
    pub fn grouped(self) -> &'static str {
        match self {
            Self::Meal => "meals",
            Self::DayPlan => "day plans",
        }
    }
}

impl fmt::Display for CategoryScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CategoryScope {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "meal" => Ok(Self::Meal),
            "day-plan" => Ok(Self::DayPlan),
            _ => Err(Error::Invalid {
                entity: Entity::Category,
                reason: format!("scope {value:?} is neither \"meal\" nor \"day-plan\""),
            }),
        }
    }
}
