//! The assistant aggregate: a plain-language request turned into typed, reviewable edits.
//!
//! The model never writes. It answers with [`Operation`] values, the rules check every one
//! of them against the real data, the user reads a [`Preview`] and confirms, and only then
//! does anything change.
//!
//! **Adding an operation costs three edits and nothing else:**
//!
//! 1. a variant on [`Operation`], and its kind on [`OperationKind`];
//! 2. an arm in `service::assistant`'s resolver, which both checks the variant and says
//!    what each date it names ends up holding;
//! 3. a tool schema in the adapter that talks to the model — `crates/anthropic`, where the
//!    schema table is a match on [`OperationKind`], so a new kind stops the workspace
//!    compiling until it has one.
//!
//! Nothing else changes, because applying and previewing are written against the resolved
//! [`DayChange`](crate::service::DayChange) rather than against the operations: an
//! operation says what it wants, the resolver says what each day becomes, and one applier
//! walks the result.

use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use super::{DayPlanItem, Macros, Unit};

/// A span of dates an operation acts on, both ends inclusive.
///
/// A single date is a range of one, so nothing downstream has to carry two shapes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DateRange {
    #[serde(with = "super::format::iso_date")]
    pub from: Date,
    #[serde(with = "super::format::iso_date")]
    pub to: Date,
}

impl DateRange {
    /// The range covering the single given date.
    pub fn just(date: Date) -> Self {
        Self {
            from: date,
            to: date,
        }
    }

    /// Every date in the range, in order. Empty when the range runs backwards.
    pub fn dates(self) -> impl Iterator<Item = Date> {
        let mut next = Some(self.from);
        std::iter::from_fn(move || {
            let date = next?;
            if date > self.to {
                next = None;
                return None;
            }
            next = date.next_day();
            Some(date)
        })
    }

    /// How many dates the range covers; zero when it runs backwards.
    pub fn length(self) -> usize {
        self.dates().count()
    }
}

/// One typed edit the assistant may propose.
///
/// Every variant names the dates it acts on, because a correction is always about days:
/// "the next four days", "tomorrow", "the rest of the week". Dates are absolute by the
/// time they get here — resolving "the next four days" against today is the model's job.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum Operation {
    /// Change how much of one product those days carry, by a signed amount in the
    /// product's own unit. The product has to be a loose item of the day's plan.
    AdjustProductAmount {
        dates: DateRange,
        product_id: Uuid,
        delta_amount: f64,
    },

    /// Put another meal or product at the end of those days' order.
    AddItem { dates: DateRange, item: DayPlanItem },

    /// Take a meal, or a loose product, out of those days' order.
    RemoveItem { dates: DateRange, item: ItemRef },

    /// Replace one meal of those days with another, in the same position.
    SwapMeal {
        dates: DateRange,
        from_meal_id: Uuid,
        to_meal_id: Uuid,
    },

    /// Put a day plan on those dates, replacing whatever they had.
    AssignPlan { dates: DateRange, day_plan_id: Uuid },

    /// Leave those dates unplanned.
    ClearPlan { dates: DateRange },
}

impl Operation {
    pub fn kind(&self) -> OperationKind {
        match self {
            Self::AdjustProductAmount { .. } => OperationKind::AdjustProductAmount,
            Self::AddItem { .. } => OperationKind::AddItem,
            Self::RemoveItem { .. } => OperationKind::RemoveItem,
            Self::SwapMeal { .. } => OperationKind::SwapMeal,
            Self::AssignPlan { .. } => OperationKind::AssignPlan,
            Self::ClearPlan { .. } => OperationKind::ClearPlan,
        }
    }

    /// The dates this operation acts on.
    pub fn dates(&self) -> DateRange {
        match self {
            Self::AdjustProductAmount { dates, .. }
            | Self::AddItem { dates, .. }
            | Self::RemoveItem { dates, .. }
            | Self::SwapMeal { dates, .. }
            | Self::AssignPlan { dates, .. }
            | Self::ClearPlan { dates } => *dates,
        }
    }
}

/// Which kind of edit an [`Operation`] is, without its arguments.
///
/// It exists so an adapter can derive one tool per kind from the enum rather than from a
/// list it keeps by hand. [`Self::ALL`] gains the new kind when [`Operation`] gains a
/// variant; the adapter's schema table is a match on this type, so the workspace does not
/// compile until the kind has a tool.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationKind {
    AdjustProductAmount,
    AddItem,
    RemoveItem,
    SwapMeal,
    AssignPlan,
    ClearPlan,
}

impl OperationKind {
    /// Every kind of operation there is.
    pub const ALL: [Self; 6] = [
        Self::AdjustProductAmount,
        Self::AddItem,
        Self::RemoveItem,
        Self::SwapMeal,
        Self::AssignPlan,
        Self::ClearPlan,
    ];

    /// The name the kind travels under: the `kind` tag of [`Operation`] on the wire, and
    /// the name of the tool an adapter derives from it. One name, so a tool call and a
    /// deserialised operation cannot disagree about which edit was meant.
    pub fn name(self) -> &'static str {
        match self {
            Self::AdjustProductAmount => "adjust-product-amount",
            Self::AddItem => "add-item",
            Self::RemoveItem => "remove-item",
            Self::SwapMeal => "swap-meal",
            Self::AssignPlan => "assign-plan",
            Self::ClearPlan => "clear-plan",
        }
    }

    /// The kind that goes by the given name, or `None` for a name no kind has.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
    }
}

/// Which item of a day's order an operation names.
///
/// A loose product is named by its product; a product inside a meal is not reachable,
/// because a meal is shared and editing one would change every plan that holds it.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum ItemRef {
    Meal { meal_id: Uuid },
    Product { product_id: Uuid },
}

/// What the assistant is told about the user's data.
///
/// Compact on purpose: the days around today with what is on them, and the labels and ids
/// of what could be put on them. No nutrient tables, no history, no weigh-ins — the model
/// only has to turn a sentence into operations over these ids.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantContext {
    #[serde(with = "super::format::iso_date")]
    pub today: Date,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Macros>,
    pub days: Vec<ContextDay>,
    /// The day plans that can be assigned. Day-local copies are left out: they are not
    /// plans the user keeps.
    pub plans: Vec<ContextPlan>,
    pub meals: Vec<ContextMeal>,
    pub products: Vec<ContextProduct>,
}

/// One date in the context: what it holds now, and what that adds up to.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextDay {
    #[serde(with = "super::format::iso_date")]
    pub date: Date,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub items: Vec<ContextItem>,
    pub macros: Macros,
}

/// One entry of a context day's order.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum ContextItem {
    Meal {
        meal_id: Uuid,
        label: String,
    },
    Product {
        product_id: Uuid,
        name: String,
        unit: Unit,
        amount: f64,
    },
}

/// A day plan the assistant may put on a date.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPlan {
    pub id: Uuid,
    pub label: String,
}

/// A meal the assistant may add to a day or swap in.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextMeal {
    pub id: Uuid,
    pub label: String,
}

/// A product the assistant may add to a day or adjust.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextProduct {
    pub id: Uuid,
    pub name: String,
    pub unit: Unit,
}

/// What the assistant answered: the operations it proposes, and why.
///
/// Nothing in it has been checked yet. It is the input to the rules, not a result.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    /// What the user typed, kept with the proposal so the confirmation and the log can
    /// show the two together.
    pub request: String,
    /// The model's own short account of what it proposed. Shown, never acted on.
    pub rationale: String,
    pub operations: Vec<Operation>,
}

/// A checked proposal, as the user reads it before confirming.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub proposal: Proposal,
    /// One entry per date the proposal would change, in date order.
    pub days: Vec<PreviewDay>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Macros>,
}

/// What one date would become.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewDay {
    #[serde(with = "super::format::iso_date")]
    pub date: Date,
    /// The plan the date would end up with, absent when it would be left unplanned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// What changes, one line per operation that touched this date, in order.
    pub changes: Vec<String>,
    pub before: Macros,
    pub after: Macros,
    /// `after` minus the targets, absent when the user has never set any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_delta: Option<Macros>,
    /// Whether applying would detach this date onto its own copy of the plan, which is
    /// what keeps every other date sharing that plan — the past included — where it is.
    pub detaches: bool,
}

/// One applied request, kept so the user can see what the assistant did and reverse it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantRecord {
    pub id: Uuid,
    pub request: String,
    pub rationale: String,
    pub operations: Vec<Operation>,
    /// One entry per date that changed, carrying what the date pointed at before, which
    /// is all it takes to put it back by hand.
    pub changes: Vec<AppliedChange>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// What an applied request did to one date.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedChange {
    #[serde(with = "super::format::iso_date")]
    pub date: Date,
    /// The plan the date pointed at before, absent when it was unplanned. Re-assigning
    /// this id, or clearing the date when it is absent, is how the change is reversed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_day_plan_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_label: Option<String>,
    /// The plan the date points at now: a day-local copy when the edit detached it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day_plan_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub detached: bool,
    pub changes: Vec<String>,
}

/// What goes into the log, which is everything but the id and the timestamp.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantRecordDraft {
    pub request: String,
    pub rationale: String,
    pub operations: Vec<Operation>,
    pub changes: Vec<AppliedChange>,
}

/// Whether the assistant has the credential it needs, without handing it back.
///
/// The key the user pastes in never leaves the device and is never returned in full; the
/// hint exists only so the user can tell which key is stored.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantKeyStatus {
    pub present: bool,
    /// The last four characters of the stored key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    #[test]
    fn every_kind_has_one_name_and_round_trips() {
        for kind in OperationKind::ALL {
            assert_eq!(OperationKind::from_name(kind.name()), Some(kind));
        }
        let names: Vec<_> = OperationKind::ALL.iter().map(|kind| kind.name()).collect();
        let unique = {
            let mut unique = names.clone();
            unique.sort_unstable();
            unique.dedup();
            unique
        };
        assert_eq!(names.len(), unique.len(), "two kinds share a name");
        assert_eq!(OperationKind::from_name("no-such-operation"), None);
    }

    #[test]
    fn an_operations_tag_is_its_kinds_name() {
        let operation = Operation::ClearPlan {
            dates: DateRange::just(date!(2026 - 10 - 20)),
        };
        let json = serde_json::to_value(&operation).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "kind": "clear-plan",
                "dates": { "from": "2026-10-20", "to": "2026-10-20" },
            })
        );
        assert_eq!(json["kind"], operation.kind().name());

        let parsed: Operation = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, operation);
    }

    #[test]
    fn a_range_walks_its_dates_and_an_inverted_one_is_empty() {
        let range = DateRange {
            from: date!(2026 - 10 - 20),
            to: date!(2026 - 10 - 23),
        };
        let dates: Vec<_> = range.dates().collect();
        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], range.from);
        assert_eq!(dates[3], range.to);
        assert_eq!(range.length(), 4);

        let inverted = DateRange {
            from: range.to,
            to: range.from,
        };
        assert_eq!(inverted.length(), 0);
        assert_eq!(DateRange::just(range.from).length(), 1);
    }
}
