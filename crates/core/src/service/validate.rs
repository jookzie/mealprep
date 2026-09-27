use crate::{
    Entity, Error, Result,
    domain::{DayPlanDraft, DayPlanItem, Macros, MealDraft, Nutrients, ProductDraft},
};

/// Checks the given product draft, trimming what the user typed.
pub(super) fn product(draft: ProductDraft) -> Result<ProductDraft> {
    let ProductDraft {
        name,
        unit,
        macros,
        nutrients,
        cost,
        brand,
    } = draft;
    let name = name.trim().to_owned();
    let nutrients: Nutrients = nutrients
        .into_iter()
        .map(|(key, value)| (key.trim().to_owned(), value))
        .filter(|(key, _)| !key.is_empty())
        .collect();
    let brand = brand
        .map(|brand| brand.trim().to_owned())
        .filter(|brand| !brand.is_empty());

    let reasons = {
        let mut reasons = Vec::new();
        if name.is_empty() {
            reasons.push("the name is empty".to_owned());
        }
        push_macro_reasons(macros, &mut reasons);
        for (key, value) in &nutrients {
            if !is_quantity(*value) {
                reasons.push(format!("nutrient {key:?} must be a non-negative number"));
            }
        }
        if cost.is_some_and(|cost| !is_quantity(cost)) {
            reasons.push("the cost must be a non-negative number".to_owned());
        }
        reasons
    };

    let draft = ProductDraft {
        name,
        unit,
        macros,
        nutrients,
        cost,
        brand,
    };
    verdict(Entity::Product, draft, reasons)
}

/// Checks the given meal draft, trimming its label.
pub(super) fn meal(draft: MealDraft) -> Result<MealDraft> {
    let MealDraft {
        label,
        category_id,
        servings,
    } = draft;
    let label = label.trim().to_owned();

    let reasons = {
        let mut reasons = Vec::new();
        if label.is_empty() {
            reasons.push("the label is empty".to_owned());
        }
        for (index, serving) in servings.iter().enumerate() {
            if !is_amount(serving.amount) {
                let position = index + 1;
                reasons.push(format!("serving {position} must have a positive amount"));
            }
        }
        reasons
    };

    let draft = MealDraft {
        label,
        category_id,
        servings,
    };
    verdict(Entity::Meal, draft, reasons)
}

/// Checks the given day plan draft, trimming its label.
pub(super) fn day_plan(draft: DayPlanDraft) -> Result<DayPlanDraft> {
    let DayPlanDraft {
        label,
        category_id,
        items,
    } = draft;
    let label = label.trim().to_owned();

    let reasons = {
        let mut reasons = Vec::new();
        if label.is_empty() {
            reasons.push("the label is empty".to_owned());
        }
        for (index, item) in items.iter().enumerate() {
            match item {
                DayPlanItem::Meal { meal_id: _ } => {}
                DayPlanItem::Product {
                    product_id: _,
                    amount,
                } => {
                    if !is_amount(*amount) {
                        let position = index + 1;
                        reasons.push(format!("item {position} must have a positive amount"));
                    }
                }
            }
        }
        reasons
    };

    let draft = DayPlanDraft {
        label,
        category_id,
        items,
    };
    verdict(Entity::DayPlan, draft, reasons)
}

/// Checks the given catalogue code, returning it trimmed.
///
/// Codes are barcodes, which are digits, but the catalogue also keys a few entries on
/// letters, so any ASCII letter or digit passes.
pub(super) fn catalogue_code(code: &str) -> Result<String> {
    let code = code.trim().to_owned();
    let reasons = if code.is_empty() || !code.chars().all(|c| c.is_ascii_alphanumeric()) {
        vec![format!("code {code:?} is not a barcode")]
    } else {
        Vec::new()
    };
    verdict(Entity::CatalogueEntry, code, reasons)
}

/// Checks the given category name, returning it trimmed.
pub(super) fn category_name(name: &str) -> Result<String> {
    let name = name.trim().to_owned();
    let reasons = if name.is_empty() {
        vec!["the name is empty".to_owned()]
    } else {
        Vec::new()
    };
    verdict(Entity::Category, name, reasons)
}

pub(super) fn targets(macros: Macros) -> Result<Macros> {
    let reasons = {
        let mut reasons = Vec::new();
        push_macro_reasons(macros, &mut reasons);
        reasons
    };
    verdict(Entity::Targets, macros, reasons)
}

/// The range a human weigh-in can plausibly fall in, in kilograms.
///
/// Wide on purpose: the check is here to catch a slipped decimal point or a pounds figure
/// typed into a kilograms field, not to have an opinion about the body on the scale.
const PLAUSIBLE_KILOGRAMS: std::ops::RangeInclusive<f64> = 20.0..=500.0;

/// Checks a weigh-in.
pub(super) fn weight(kilograms: f64) -> Result<f64> {
    let mut reasons = Vec::new();
    if !is_amount(kilograms) {
        reasons.push("the weight must be a positive number".to_owned());
    } else if !PLAUSIBLE_KILOGRAMS.contains(&kilograms) {
        reasons.push(format!(
            "{kilograms} kg is outside the plausible range of {} to {} kg",
            PLAUSIBLE_KILOGRAMS.start(),
            PLAUSIBLE_KILOGRAMS.end()
        ));
    }
    verdict(Entity::WeightEntry, kilograms, reasons)
}

fn push_macro_reasons(macros: Macros, reasons: &mut Vec<String>) {
    for (figure, value) in macros.named() {
        if !is_quantity(value) {
            reasons.push(format!("{figure} must be a non-negative number"));
        }
    }
}

fn is_quantity(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn is_amount(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn verdict<T>(entity: Entity, value: T, reasons: Vec<String>) -> Result<T> {
    if reasons.is_empty() {
        return Ok(value);
    }
    let reason = reasons.join("; ");
    Err(Error::Invalid { entity, reason })
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::domain::{Serving, Unit};

    #[test]
    fn trims_a_product_and_drops_blank_fields() {
        let draft = ProductDraft {
            name: "  Oats ".to_owned(),
            unit: Unit::Gram,
            macros: Macros::ZERO,
            nutrients: [(" fiber ".to_owned(), 10.0), ("  ".to_owned(), 1.0)].into(),
            cost: None,
            brand: Some("   ".to_owned()),
        };

        let checked = product(draft).unwrap();
        assert_eq!(checked.name, "Oats");
        assert_eq!(checked.nutrients, [("fiber".to_owned(), 10.0)].into());
        assert_eq!(checked.brand, None);
    }

    #[test]
    fn reports_every_problem_with_a_product_at_once() {
        let draft = ProductDraft {
            name: String::new(),
            unit: Unit::Millilitre,
            macros: Macros {
                fat_g: -1.0,
                ..Macros::ZERO
            },
            nutrients: Nutrients::new(),
            cost: Some(f64::NAN),
            brand: None,
        };

        let Err(Error::Invalid { entity, reason }) = product(draft) else {
            panic!("the draft should be invalid");
        };
        assert_eq!(entity, Entity::Product);
        assert_eq!(
            reason,
            "the name is empty; fat must be a non-negative number; the cost must be a non-negative number"
        );
    }

    #[test]
    fn accepts_a_trimmed_barcode_and_nothing_that_could_leave_a_url_path() {
        assert_eq!(catalogue_code(" 3017620422003 ").unwrap(), "3017620422003");
        for code in ["", "   ", "301/../x", "3017 6204", "٣٠١٧"] {
            assert!(catalogue_code(code).is_err(), "{code:?} should be rejected");
        }
    }

    #[test]
    fn rejects_empty_servings() {
        let draft = MealDraft {
            label: "Lunch".to_owned(),
            category_id: None,
            servings: vec![Serving {
                product_id: Uuid::new_v4(),
                amount: 0.0,
            }],
        };
        assert!(meal(draft).is_err());
    }
}
