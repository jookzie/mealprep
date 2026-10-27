//! One tool per kind of operation, and the way back from a tool call to the operation.
//!
//! [`schema`] is a match on [`OperationKind`], so a new kind of operation stops this crate
//! compiling until it has been given a tool. That is the whole mechanism: there is no list
//! of tool names anywhere for a new operation to be left out of.
//!
//! A tool's input is the operation's own serialised shape with the `kind` tag taken out, so
//! [`operation`] puts the tag back and lets the domain's own `Deserialize` do the rest. One
//! shape, defined once, for the wire and for the model.

use mealprep_core::{
    Entity, Error, Result,
    domain::{Operation, OperationKind},
};
use serde_json::{Value, json};

/// The tools the model may call, one per kind of operation.
pub(crate) fn tools() -> Vec<Value> {
    OperationKind::ALL.into_iter().map(schema).collect()
}

/// Turns one tool call into the operation it names.
///
/// The arguments are the model's, so nothing here is trusted beyond its shape: the amounts,
/// the ids and the dates are all checked against the user's real data by the rules.
pub(crate) fn operation(name: &str, mut input: Value) -> Result<Operation> {
    let kind = OperationKind::from_name(name).ok_or_else(|| invalid(format!("{name:?}")))?;
    let Some(fields) = input.as_object_mut() else {
        return Err(invalid(format!("{name} with {input}")));
    };
    fields.insert("kind".to_owned(), Value::String(kind.name().to_owned()));
    serde_json::from_value(input).map_err(|source| invalid(format!("{name}: {source}")))
}

fn invalid(what: String) -> Error {
    Error::Invalid {
        entity: Entity::AssistantRequest,
        reason: format!("the assistant proposed something unreadable: {what}"),
    }
}

/// The tool for one kind of operation: what it does, and the arguments it takes.
///
/// The descriptions are the model's only documentation, so they say what the edit means
/// rather than restating the field names.
fn schema(kind: OperationKind) -> Value {
    let (description, properties): (&str, Value) = match kind {
        OperationKind::AdjustProductAmount => (
            "Change how much of one product those days carry, by a signed amount in the \
             product's own unit. The product has to already be an item of the day's own \
             order: a product inside a meal is shared with every other day on that meal \
             and cannot be changed for one day.",
            json!({
                "dates": dates(),
                "productId": {
                    "type": "string",
                    "description": "The id of the product, from the context's products.",
                },
                "deltaAmount": {
                    "type": "number",
                    "description": "How much to add, in the product's unit; negative to take some away. \
                        The result has to stay above zero — remove the item instead of taking all of it.",
                },
            }),
        ),

        OperationKind::AddItem => (
            "Put another meal, or a product on its own, at the end of those days' order.",
            json!({ "dates": dates(), "item": day_plan_item() }),
        ),

        OperationKind::RemoveItem => (
            "Take a meal, or a product that is an item of its own, out of those days' order.",
            json!({ "dates": dates(), "item": item_ref() }),
        ),

        OperationKind::SwapMeal => (
            "Replace one meal of those days with another, in the same position.",
            json!({
                "dates": dates(),
                "fromMealId": {
                    "type": "string",
                    "description": "The id of the meal that is on those days now.",
                },
                "toMealId": {
                    "type": "string",
                    "description": "The id of the meal to put in its place, from the context's meals.",
                },
            }),
        ),

        OperationKind::AssignPlan => (
            "Put a whole day plan on those dates, replacing whatever they had. Use this when \
             the request is about which plan a day follows rather than about one item of it.",
            json!({
                "dates": dates(),
                "dayPlanId": {
                    "type": "string",
                    "description": "The id of the plan, from the context's plans.",
                },
            }),
        ),

        OperationKind::ClearPlan => (
            "Leave those dates with no plan at all. Only for a day that is genuinely not \
             being planned; a day that changes still has a plan.",
            json!({ "dates": dates() }),
        ),

        OperationKind::CreateProduct => (
            "Add a product to the user's list, with its figures per 100 g or ml. Only when \
             no product in the context is the one meant; reuse an existing one otherwise.",
            json!({ "newId": new_id("product"), "product": product() }),
        ),

        OperationKind::UpdateProduct => (
            "Change some of a product's fields; leave out what stays. Every meal and day \
             plan that holds the product follows, on every date.",
            json!({
                "productId": {
                    "type": "string",
                    "description": "The id of the product, from the context's products.",
                },
                "changes": product_patch(),
            }),
        ),

        OperationKind::CreateMeal => (
            "Add a meal to the user's list: a named set of products with amounts. A meal \
             is shared by every day plan that holds it.",
            json!({
                "newId": new_id("meal"),
                "meal": {
                    "type": "object",
                    "properties": {
                        "label": { "type": "string", "description": "What the meal is called." },
                        "servings": servings(),
                    },
                    "required": ["label", "servings"],
                },
            }),
        ),

        OperationKind::UpdateMeal => (
            "Rename a meal or replace what it is made of; leave out what stays. Every day \
             plan that holds the meal follows, on every date, past ones included. When the \
             request is about some dates only, edit those dates instead.",
            json!({
                "mealId": {
                    "type": "string",
                    "description": "The id of the meal, from the context's meals.",
                },
                "changes": {
                    "type": "object",
                    "properties": {
                        "label": { "type": "string", "description": "The new name." },
                        "servings": servings(),
                    },
                },
            }),
        ),

        OperationKind::CreateDayPlan => (
            "Add a day plan to the user's list: a named day of meals and products, in \
             order, that dates can then be put on.",
            json!({
                "newId": new_id("day plan"),
                "dayPlan": {
                    "type": "object",
                    "properties": {
                        "label": { "type": "string", "description": "What the plan is called." },
                        "items": items(),
                    },
                    "required": ["label", "items"],
                },
            }),
        ),

        OperationKind::UpdateDayPlan => (
            "Rename a day plan or replace its items; leave out what stays. Every date on \
             the plan follows, past ones included. When the request is about some dates \
             only, edit those dates instead.",
            json!({
                "dayPlanId": {
                    "type": "string",
                    "description": "The id of the plan, from the context's plans.",
                },
                "changes": {
                    "type": "object",
                    "properties": {
                        "label": { "type": "string", "description": "The new name." },
                        "items": items(),
                    },
                },
            }),
        ),
    };

    let required: Vec<&str> = properties
        .as_object()
        .expect("internal error: a tool's properties are always an object")
        .keys()
        .map(String::as_str)
        .collect();

    json!({
        "name": kind.name(),
        "description": description,
        "input_schema": {
            "type": "object",
            "properties": properties,
            "required": required,
        },
    })
}

fn dates() -> Value {
    json!({
        "type": "object",
        "description": "The dates to change, both ends included. One day is the same date in both.",
        "properties": {
            "from": { "type": "string", "description": "The first date, as YYYY-MM-DD." },
            "to": { "type": "string", "description": "The last date, as YYYY-MM-DD." },
        },
        "required": ["from", "to"],
    })
}

/// The id a create gives what it makes, so the rest of the same answer can name it.
fn new_id(what: &str) -> Value {
    json!({
        "type": "string",
        "description": format!(
            "A new random UUID for the {what}, unused in the context. Other calls of the same \
             answer name the new {what} by it; the app replaces it with its own when applying."
        ),
    })
}

/// A whole product, as the product form takes it.
fn product() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "brand": { "type": "string", "description": "Only when known." },
            "unit": {
                "type": "string",
                "enum": ["g", "ml"],
                "description": "Whether the figures are per 100 g or per 100 ml.",
            },
            "macros": {
                "type": "object",
                "description": "Per 100 of the unit, as on a typical package.",
                "properties": {
                    "energyKcal": { "type": "number" },
                    "proteinG": { "type": "number" },
                    "fatG": { "type": "number" },
                    "carbohydratesG": { "type": "number" },
                },
                "required": ["energyKcal", "proteinG", "fatG", "carbohydratesG"],
            },
            "cost": { "type": "number", "description": "Price per 100 of the unit, only when the user gave one." },
        },
        "required": ["name", "unit", "macros"],
    })
}

/// The fields of a product an update may change, every one optional.
fn product_patch() -> Value {
    json!({
        "type": "object",
        "description": "Only the fields that change.",
        "properties": {
            "name": { "type": "string" },
            "brand": { "type": "string" },
            "energyKcal": { "type": "number", "description": "Per 100 of the product's unit." },
            "proteinG": { "type": "number", "description": "Per 100 of the product's unit." },
            "fatG": { "type": "number", "description": "Per 100 of the product's unit." },
            "carbohydratesG": { "type": "number", "description": "Per 100 of the product's unit." },
            "cost": { "type": "number", "description": "Price per 100 of the product's unit." },
        },
    })
}

/// What a meal is made of.
fn servings() -> Value {
    json!({
        "type": "array",
        "description": "Every product of the meal with its amount; given, it replaces them all.",
        "items": {
            "type": "object",
            "properties": {
                "productId": { "type": "string", "description": "A product's id, or a newId created in this answer." },
                "amount": { "type": "number", "description": "In the product's own unit." },
            },
            "required": ["productId", "amount"],
        },
    })
}

/// What a day plan holds, in order.
fn items() -> Value {
    json!({
        "type": "array",
        "description": "Every entry of the day in order; given, it replaces them all.",
        "items": day_plan_item(),
    })
}

/// An entry of a day's order, as the day plans themselves carry it.
fn day_plan_item() -> Value {
    json!({
        "type": "object",
        "description": "A whole meal, or a product with how much of it.",
        "properties": {
            "kind": { "type": "string", "enum": ["meal", "product"] },
            "mealId": { "type": "string", "description": "The meal's id. Only when kind is meal." },
            "productId": { "type": "string", "description": "The product's id. Only when kind is product." },
            "amount": {
                "type": "number",
                "description": "How much of the product, in its own unit. Only when kind is product.",
            },
        },
        "required": ["kind"],
    })
}

/// Which entry of a day's order an operation names; an amount would mean nothing here.
fn item_ref() -> Value {
    json!({
        "type": "object",
        "description": "Which entry to act on: a meal, or a product that is an item of its own.",
        "properties": {
            "kind": { "type": "string", "enum": ["meal", "product"] },
            "mealId": { "type": "string", "description": "The meal's id. Only when kind is meal." },
            "productId": { "type": "string", "description": "The product's id. Only when kind is product." },
        },
        "required": ["kind"],
    })
}

#[cfg(test)]
mod tests {
    use mealprep_core::domain::{
        DateRange, DayPlanItem, ItemRef, MealDraft, ProductPatch, Serving,
    };
    use uuid::Uuid;

    use super::*;

    #[test]
    fn there_is_one_tool_per_kind_of_operation() {
        let tools = tools();
        assert_eq!(tools.len(), OperationKind::ALL.len());

        for (tool, kind) in tools.iter().zip(OperationKind::ALL) {
            assert_eq!(tool["name"], kind.name());
            assert!(
                tool["description"]
                    .as_str()
                    .is_some_and(|text| !text.is_empty()),
                "{} has no description",
                kind.name()
            );
            // Every argument is required: an operation with a field missing is not an
            // operation, and the rules would refuse it a step later anyway.
            let properties = tool["input_schema"]["properties"].as_object().unwrap();
            let required = tool["input_schema"]["required"].as_array().unwrap();
            assert_eq!(properties.len(), required.len(), "{}", kind.name());
            // A day edit names dates; an entity edit names none.
            let entity_edit =
                kind.name().starts_with("create-") || kind.name().starts_with("update-");
            assert_eq!(
                properties.contains_key("dates"),
                !entity_edit,
                "{}",
                kind.name()
            );
        }
    }

    #[test]
    fn a_tool_call_becomes_the_operation_it_names() {
        let product_id = Uuid::new_v4();
        let dates = DateRange {
            from: mealprep_core::domain::parse_iso_date("2026-10-20").unwrap(),
            to: mealprep_core::domain::parse_iso_date("2026-10-23").unwrap(),
        };

        let decoded = operation(
            "adjust-product-amount",
            json!({
                "dates": { "from": "2026-10-20", "to": "2026-10-23" },
                "productId": product_id,
                "deltaAmount": -40.0,
            }),
        )
        .unwrap();
        assert_eq!(
            decoded,
            Operation::AdjustProductAmount {
                dates,
                product_id,
                delta_amount: -40.0,
            }
        );

        let meal_id = Uuid::new_v4();
        let added = operation(
            "add-item",
            json!({
                "dates": { "from": "2026-10-20", "to": "2026-10-20" },
                "item": { "kind": "meal", "mealId": meal_id },
            }),
        )
        .unwrap();
        assert_eq!(
            added,
            Operation::AddItem {
                dates: DateRange::just(dates.from),
                item: DayPlanItem::Meal { meal_id },
            }
        );

        let removed = operation(
            "remove-item",
            json!({
                "dates": { "from": "2026-10-20", "to": "2026-10-20" },
                "item": { "kind": "product", "productId": product_id },
            }),
        )
        .unwrap();
        assert_eq!(
            removed,
            Operation::RemoveItem {
                dates: DateRange::just(dates.from),
                item: ItemRef::Product { product_id },
            }
        );
    }

    #[test]
    fn an_entity_tool_call_becomes_the_operation_it_names() {
        let new_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let created = operation(
            "create-meal",
            json!({
                "newId": new_id,
                "meal": {
                    "label": "Oat bowl",
                    "servings": [{ "productId": product_id, "amount": 80 }],
                },
            }),
        )
        .unwrap();
        assert_eq!(
            created,
            Operation::CreateMeal {
                new_id,
                meal: MealDraft {
                    label: "Oat bowl".to_owned(),
                    category_id: None,
                    servings: vec![Serving {
                        product_id,
                        amount: 80.0,
                    }],
                },
            }
        );

        // A patch takes only what changes; the rest stays absent rather than zeroed.
        let updated = operation(
            "update-product",
            json!({ "productId": product_id, "changes": { "energyKcal": 400 } }),
        )
        .unwrap();
        assert_eq!(
            updated,
            Operation::UpdateProduct {
                product_id,
                changes: ProductPatch {
                    energy_kcal: Some(400.0),
                    ..ProductPatch::default()
                },
            }
        );

        let product = operation(
            "create-product",
            json!({
                "newId": new_id,
                "product": {
                    "name": "Oats",
                    "unit": "g",
                    "macros": { "energyKcal": 370, "proteinG": 13, "fatG": 7, "carbohydratesG": 60 },
                },
            }),
        )
        .unwrap();
        assert!(matches!(product, Operation::CreateProduct { .. }));
    }

    #[test]
    fn an_unreadable_tool_call_is_invalid_rather_than_guessed_at() {
        for (name, input) in [
            ("no-such-tool", json!({})),
            ("clear-plan", json!("not an object")),
            (
                "clear-plan",
                json!({ "dates": { "from": "the 20th", "to": "the 23rd" } }),
            ),
            (
                "assign-plan",
                json!({ "dates": { "from": "2026-10-20", "to": "2026-10-20" } }),
            ),
            (
                "add-item",
                json!({
                    "dates": { "from": "2026-10-20", "to": "2026-10-20" },
                    "item": { "kind": "product", "productId": Uuid::new_v4() },
                }),
            ),
        ] {
            assert!(
                matches!(operation(name, input.clone()), Err(Error::Invalid { .. })),
                "{name} with {input} should be refused"
            );
        }
    }
}
