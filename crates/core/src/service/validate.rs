use time::Date;

use crate::{
    Entity, Error, Result,
    domain::{
        DayPlanDraft, DayPlanItem, Macros, MealDraft, Nutrients, ProductDraft, parse_iso_date,
    },
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

/// The longest request the assistant takes. A correction is a sentence; anything this
/// long is a document, and sending it would only cost the user tokens.
const MAX_REQUEST_CHARS: usize = 2_000;

/// Checks what the user typed at the assistant, returning it trimmed.
pub(super) fn assistant_request(request: String) -> Result<String> {
    let request = request.trim().to_owned();
    let reasons = if request.is_empty() {
        vec!["the request is empty".to_owned()]
    } else if request.chars().count() > MAX_REQUEST_CHARS {
        vec![format!(
            "the request is longer than {MAX_REQUEST_CHARS} characters"
        )]
    } else {
        Vec::new()
    };
    verdict(Entity::AssistantRequest, request, reasons)
}

/// Checks the credential the user pasted in, returning it trimmed.
///
/// What a valid key looks like is the adapter's business; all that is checked here is
/// that it is one token, because a key with a space in it is a paste that went wrong.
pub(super) fn assistant_key(key: String) -> Result<String> {
    let key = key.trim().to_owned();
    let reasons = if key.is_empty() {
        vec!["the key is empty".to_owned()]
    } else if key.chars().any(|c| c.is_whitespace() || c.is_control()) {
        vec!["the key has a space or a line break in it".to_owned()]
    } else {
        Vec::new()
    };
    verdict(Entity::AssistantRequest, key, reasons)
}

/// The words that ask for a day already gone and cannot mean anything else.
///
/// Deliberately short. Everything outside it counts as not asking for the past, so a
/// request that merely sounds retrospective is refused rather than rewriting history —
/// which is the failure that cannot be undone.
const PAST_PHRASES: [&str; 13] = [
    "yesterday",
    "last week",
    "last month",
    "last monday",
    "last tuesday",
    "last wednesday",
    "last thursday",
    "last friday",
    "last saturday",
    "last sunday",
    "earlier this week",
    "so far this week",
    "in the past",
];

/// Whether the request unambiguously asks for a date before today.
///
/// It does when it names a `YYYY-MM-DD` date that has passed, or uses one of the phrases
/// above. The rules refuse to touch a past date otherwise.
pub(super) fn past_dates_requested(request: &str, today: Date) -> bool {
    let lowercase = request.to_lowercase();
    if PAST_PHRASES.iter().any(|phrase| lowercase.contains(phrase)) {
        return true;
    }
    mentioned_dates(&lowercase)
        .into_iter()
        .any(|date| date < today)
}

/// Every `YYYY-MM-DD` date the text spells out.
///
/// Split on anything a date cannot contain, so a date keeps its punctuation out: "on
/// 2026-10-14." yields the date and not the full stop.
fn mentioned_dates(text: &str) -> Vec<Date> {
    text.split(|c: char| !c.is_ascii_digit() && c != '-')
        .filter_map(|token| parse_iso_date(token).ok())
        .collect()
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
    fn reads_a_past_date_only_when_the_request_is_unambiguous() {
        let today = crate::domain::parse_iso_date("2026-10-20").unwrap();

        for request in [
            "yesterday I had 40 g less whey",
            "Last Friday was a rest day",
            "on 2026-10-14 I skipped lunch",
            "drop the chocolate in the past week",
        ] {
            assert!(
                past_dates_requested(request, today),
                "{request:?} asks for the past"
            );
        }

        for request in [
            "next 4 days I have 40 g less whey",
            "add a banana tomorrow",
            "on 2026-10-25 I am travelling",
            "swap the last meal of the day",
        ] {
            assert!(
                !past_dates_requested(request, today),
                "{request:?} does not ask for the past"
            );
        }
    }

    #[test]
    fn trims_a_request_and_a_key_and_refuses_the_useless_ones() {
        assert_eq!(
            assistant_request("  more protein  ".to_owned()).unwrap(),
            "more protein"
        );
        assert!(assistant_request("   ".to_owned()).is_err());
        assert!(assistant_request("x".repeat(2_001)).is_err());

        assert_eq!(
            assistant_key(" sk-ant-demo ".to_owned()).unwrap(),
            "sk-ant-demo"
        );
        assert!(assistant_key("sk-ant demo".to_owned()).is_err());
        assert!(assistant_key(String::new()).is_err());
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
