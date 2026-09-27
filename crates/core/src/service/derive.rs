use std::collections::HashMap;

use uuid::Uuid;

use crate::domain::{
    Cost, DayPlan, DayPlanItem, Derived, LooseProduct, Macros, Meal, Product, ResolvedDayPlan,
    ResolvedDayPlanItem, Serving,
};

/// Products by id, as every derivation joins against them.
pub(super) type ProductIndex = HashMap<Uuid, Product>;

/// Meals by id, with their own figures already derived.
pub(super) type MealIndex = HashMap<Uuid, Derived<Meal>>;

pub(super) fn index_products(products: Vec<Product>) -> ProductIndex {
    products
        .into_iter()
        .map(|product| (product.id, product))
        .collect()
}

/// Derives what the given serving contributes to macros.
///
/// A product's macros are per 100 units and the amount is in those same units, so the
/// serving is the product scaled by amount/100. A deleted product contributes nothing.
pub(super) fn serving_macros(serving: &Serving, products: &ProductIndex) -> Macros {
    products
        .get(&serving.product_id)
        .map_or(Macros::ZERO, |product| {
            product.macros.scale(serving.amount / 100.0)
        })
}

/// Derives what the given serving costs, on the same terms as [`serving_macros`].
///
/// A product without a price, or one that has been deleted, adds nothing and makes the
/// cost incomplete: its price is unknowable either way.
pub(super) fn serving_cost(serving: &Serving, products: &ProductIndex) -> Cost {
    let price = products
        .get(&serving.product_id)
        .and_then(|product| product.cost);
    let Some(price) = price else {
        return Cost::UNKNOWN;
    };
    Cost {
        amount: price * serving.amount / 100.0,
        complete: true,
    }
}

pub(super) fn derive_meal(meal: Meal, products: &ProductIndex) -> Derived<Meal> {
    let macros = meal
        .servings
        .iter()
        .map(|serving| serving_macros(serving, products))
        .sum();
    let cost = meal
        .servings
        .iter()
        .map(|serving| serving_cost(serving, products))
        .sum();
    Derived {
        entity: meal,
        macros,
        cost,
    }
}

/// Resolves the given plan's items and sums them.
///
/// A deleted product still counts as an entry, because its amount is what the user wrote
/// down; a deleted meal drops out.
pub(super) fn resolve_day_plan(
    plan: DayPlan,
    meals: &MealIndex,
    products: &ProductIndex,
) -> ResolvedDayPlan {
    let DayPlan {
        id,
        label,
        category_id,
        items,
        created_at,
        updated_at,
    } = plan;

    let items: Vec<_> = items
        .into_iter()
        .filter_map(|item| resolve_item(item, meals, products))
        .collect();
    let macros = items.iter().map(ResolvedDayPlanItem::macros).sum();
    let cost = items.iter().map(ResolvedDayPlanItem::cost).sum();

    ResolvedDayPlan {
        id,
        label,
        category_id,
        items,
        macros,
        cost,
        created_at,
        updated_at,
    }
}

fn resolve_item(
    item: DayPlanItem,
    meals: &MealIndex,
    products: &ProductIndex,
) -> Option<ResolvedDayPlanItem> {
    match item {
        DayPlanItem::Meal { meal_id } => {
            let meal = meals.get(&meal_id)?.clone();
            Some(ResolvedDayPlanItem::Meal { meal })
        }
        DayPlanItem::Product { product_id, amount } => {
            let serving = Serving { product_id, amount };
            let found = products.get(&product_id);
            let product = LooseProduct {
                id: product_id,
                name: found.map(|found| found.name.clone()),
                unit: found.map(|found| found.unit),
                amount,
                macros: serving_macros(&serving, products),
                cost: serving_cost(&serving, products),
            };
            Some(ResolvedDayPlanItem::Product { product })
        }
    }
}

/// Collects the distinct products the given servings name, in first-seen order.
pub(super) fn product_ids<'serving>(
    servings: impl IntoIterator<Item = &'serving Serving>,
) -> Vec<Uuid> {
    let mut ids = Vec::new();
    for serving in servings {
        push_distinct(&mut ids, serving.product_id);
    }
    ids
}

/// Collects the distinct meals and loose products the given items name, in first-seen order.
pub(super) fn item_ids<'item>(
    items: impl IntoIterator<Item = &'item DayPlanItem>,
) -> (Vec<Uuid>, Vec<Uuid>) {
    let mut meal_ids = Vec::new();
    let mut product_ids = Vec::new();
    for item in items {
        match item {
            DayPlanItem::Meal { meal_id } => push_distinct(&mut meal_ids, *meal_id),
            DayPlanItem::Product {
                product_id,
                amount: _,
            } => push_distinct(&mut product_ids, *product_id),
        }
    }
    (meal_ids, product_ids)
}

fn push_distinct(ids: &mut Vec<Uuid>, id: Uuid) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use super::*;
    use crate::domain::{Nutrients, Unit};

    // test types

    fn product(cost: Option<f64>) -> Product {
        Product {
            id: Uuid::new_v4(),
            name: "Oats".to_owned(),
            unit: Unit::Gram,
            macros: Macros {
                energy_kcal: 380.0,
                fat_g: 7.0,
                protein_g: 13.0,
                carbohydrates_g: 60.0,
            },
            nutrients: Nutrients::new(),
            cost,
            brand: None,
            source_code: None,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }

    fn meal(servings: Vec<Serving>) -> Meal {
        Meal {
            id: Uuid::new_v4(),
            label: "Breakfast".to_owned(),
            category_id: None,
            servings,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn scales_a_serving_by_its_amount_per_hundred() {
        let oats = product(Some(0.5));
        let serving = Serving {
            product_id: oats.id,
            amount: 50.0,
        };
        let products = index_products(vec![oats]);

        assert_eq!(serving_macros(&serving, &products).energy_kcal, 190.0);
        assert_eq!(
            serving_cost(&serving, &products),
            Cost {
                amount: 0.25,
                complete: true
            }
        );
    }

    #[test]
    fn an_unpriced_or_deleted_product_makes_the_meal_cost_incomplete() {
        let priced = product(Some(1.0));
        let unpriced = product(None);
        let deleted = Uuid::new_v4();
        let servings = vec![
            Serving {
                product_id: priced.id,
                amount: 200.0,
            },
            Serving {
                product_id: unpriced.id,
                amount: 100.0,
            },
            Serving {
                product_id: deleted,
                amount: 100.0,
            },
        ];
        let products = index_products(vec![priced, unpriced]);

        let derived = derive_meal(meal(servings), &products);
        assert_eq!(
            derived.cost,
            Cost {
                amount: 2.0,
                complete: false
            }
        );
        assert_eq!(derived.macros.energy_kcal, 380.0 * 3.0);
    }

    #[test]
    fn a_day_plan_keeps_deleted_products_and_drops_deleted_meals() {
        let oats = product(Some(1.0));
        let breakfast = meal(vec![Serving {
            product_id: oats.id,
            amount: 100.0,
        }]);
        let gone_product = Uuid::new_v4();
        let plan = DayPlan {
            id: Uuid::new_v4(),
            label: "Rest day".to_owned(),
            category_id: None,
            items: vec![
                DayPlanItem::Meal {
                    meal_id: breakfast.id,
                },
                DayPlanItem::Meal {
                    meal_id: Uuid::new_v4(),
                },
                DayPlanItem::Product {
                    product_id: gone_product,
                    amount: 30.0,
                },
            ],
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        };
        let products = index_products(vec![oats]);
        let meals: MealIndex = [(breakfast.id, derive_meal(breakfast, &products))].into();

        let resolved = resolve_day_plan(plan, &meals, &products);
        assert_eq!(resolved.items.len(), 2);
        assert_eq!(resolved.macros.energy_kcal, 380.0);
        assert!(!resolved.cost.complete);
        assert!(matches!(
            &resolved.items[1],
            ResolvedDayPlanItem::Product { product } if product.id == gone_product && product.name.is_none()
        ));
    }

    #[test]
    fn collects_distinct_ids_in_order() {
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let items = [
            DayPlanItem::Meal { meal_id: first },
            DayPlanItem::Meal { meal_id: first },
            DayPlanItem::Product {
                product_id: second,
                amount: 1.0,
            },
        ];
        assert_eq!(item_ids(&items), (vec![first], vec![second]));
    }
}
