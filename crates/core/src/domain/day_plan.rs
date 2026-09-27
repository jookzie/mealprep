use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{Cost, Derived, Macros, Meal, Unit};

/// A labelled, ordered list of what is eaten in a day, optionally in one category.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayPlan {
    pub id: Uuid,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    pub items: Vec<DayPlanItem>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// What the user writes when creating or replacing a day plan.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayPlanDraft {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    pub items: Vec<DayPlanItem>,
}

/// One entry in a day plan's order: a whole meal, or a product eaten on its own.
///
/// Both share one sequence, because a day is eaten in one order.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum DayPlanItem {
    Meal { meal_id: Uuid },
    Product { product_id: Uuid, amount: f64 },
}

/// A day plan with what its items name filled in and its totals derived.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedDayPlan {
    pub id: Uuid,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    pub items: Vec<ResolvedDayPlanItem>,
    pub macros: Macros,
    pub cost: Cost,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// A [`DayPlanItem`] with what it names filled in.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ResolvedDayPlanItem {
    Meal { meal: Derived<Meal> },
    Product { product: LooseProduct },
}

impl ResolvedDayPlanItem {
    /// Returns what this item contributes to the plan's macros.
    pub fn macros(&self) -> Macros {
        match self {
            Self::Meal { meal } => meal.macros,
            Self::Product { product } => product.macros,
        }
    }

    /// Returns what this item contributes to the plan's cost.
    pub fn cost(&self) -> Cost {
        match self {
            Self::Meal { meal } => meal.cost,
            Self::Product { product } => product.cost,
        }
    }
}

/// A product eaten on its own within a day plan.
///
/// `name` and `unit` are absent when the product has since been deleted; the entry still
/// renders, because its amount is what the user wrote down.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LooseProduct {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<Unit>,
    pub amount: f64,
    pub macros: Macros,
    pub cost: Cost,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_are_tagged_by_kind() {
        let meal_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let items = vec![
            DayPlanItem::Meal { meal_id },
            DayPlanItem::Product {
                product_id,
                amount: 150.0,
            },
        ];

        let json = serde_json::to_value(&items).unwrap();
        assert_eq!(
            json,
            serde_json::json!([
                { "kind": "meal", "mealId": meal_id },
                { "kind": "product", "productId": product_id, "amount": 150.0 },
            ])
        );

        let parsed: Vec<DayPlanItem> = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, items);
    }
}
