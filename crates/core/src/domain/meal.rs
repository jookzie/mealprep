use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// A labelled, ordered set of products with serving sizes, optionally in one category.
///
/// Only the composition is stored; its macros and cost are derived on read.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meal {
    pub id: Uuid,
    pub label: String,
    /// Absent when the meal is in no category. The category may since have been deleted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    pub servings: Vec<Serving>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// What the user writes when creating or replacing a meal.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MealDraft {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    pub servings: Vec<Serving>,
}

/// A product in a composition, with the amount in the product's own unit.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Serving {
    pub product_id: Uuid,
    pub amount: f64,
}
