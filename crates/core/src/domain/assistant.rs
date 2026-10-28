//! The assistant aggregate: a plain-language request turned into typed, reviewable edits.
//!
//! The model never writes. It answers with [`Operation`] values, the rules check every one
//! of them against the real data, the user reads a [`Preview`] and confirms, and only then
//! does anything change.
//!
//! Two families of operation. Most are about days — "the next four days", "tomorrow" — and
//! name the dates they act on. The rest create or change a product, a meal, a day plan or
//! the targets themselves, name no date, and reach every date that uses what they change.
//!
//! **Adding an operation costs three edits and nothing else:**
//!
//! 1. a variant on [`Operation`], and its kind on [`OperationKind`];
//! 2. an arm in `service::assistant`'s resolver, which both checks the variant and says
//!    what each date it names, or each entity it touches, ends up holding;
//! 3. a tool schema in the adapter that talks to the model — `crates/anthropic`, where the
//!    schema table is a match on [`OperationKind`], so a new kind stops the workspace
//!    compiling until it has one.
//!
//! Nothing else changes, because applying and previewing are written against the resolved
//! [`DayChange`](crate::service::DayChange) rather than against the operations: an
//! operation says what it wants, the resolver says what each day becomes, and one applier
//! walks the result.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use super::{
    CategoryScope, DayPlanDraft, DayPlanItem, Macros, MealDraft, Nutrients, ProductDraft, Serving,
    Unit,
};

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
/// A day edit names the dates it acts on: "the next four days", "tomorrow", "the rest of
/// the week". Dates are absolute by the time they get here — resolving "the next four
/// days" against today is the model's job. An entity edit names no date: it creates or
/// changes a product, meal or day plan, and so every date that uses it.
///
/// A create carries a `new_id` the model picks, so later operations of the same proposal
/// can name what it creates. The id is provisional: the store assigns the real one when
/// the proposal is applied, and the operations after it are pointed at that.
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

    /// Add a product to the user's list, figures per 100 g or ml.
    CreateProduct { new_id: Uuid, product: ProductDraft },

    /// Change some of a product's fields. Every meal and plan that holds it follows.
    UpdateProduct {
        product_id: Uuid,
        changes: ProductPatch,
    },

    /// Add a meal to the user's list.
    CreateMeal { new_id: Uuid, meal: MealDraft },

    /// Rename a meal or replace its servings. Every plan that holds it follows.
    UpdateMeal { meal_id: Uuid, changes: MealPatch },

    /// Add a day plan to the user's list, for assigning to dates.
    CreateDayPlan {
        new_id: Uuid,
        day_plan: DayPlanDraft,
    },

    /// Rename a day plan or replace its items. Every date on it follows, past ones too.
    UpdateDayPlan {
        day_plan_id: Uuid,
        changes: DayPlanPatch,
    },

    /// Replace a product's figures with the catalogue's current ones, by the code it was
    /// imported from or the one given. The figures are fetched when the proposal is
    /// checked, so the model never has to know them.
    SyncProduct {
        product_id: Uuid,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },

    /// Change some of the daily targets.
    UpdateTargets { changes: TargetsPatch },
}

/// The fields of a product an update changes; a field left out keeps its value, and a
/// field a product can lack is removed by `null`.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        default,
        deserialize_with = "super::format::nullable::deserialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub brand: Option<Option<String>>,
    /// What the figures are per 100 of. Changing it reinterprets every amount already
    /// served of the product, so it is for correcting a product entered wrongly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<Unit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_kcal: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protein_g: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fat_g: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carbohydrates_g: Option<f64>,
    /// Per 100 g or ml.
    #[serde(
        default,
        deserialize_with = "super::format::nullable::deserialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub cost: Option<Option<f64>>,
    /// The nutrients beyond the macros that change, per 100 of the unit and keyed as the
    /// product keys them; `null` removes one. Those not named keep their values.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub nutrients: BTreeMap<String, Option<f64>>,
}

/// The parts of a meal an update changes; the servings, when given, replace them all, and
/// a `null` category takes it out of the one it is in.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MealPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(
        default,
        deserialize_with = "super::format::nullable::deserialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub category_id: Option<Option<Uuid>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub servings: Option<Vec<Serving>>,
}

/// The parts of a day plan an update changes; the items, when given, replace them all, and
/// a `null` category takes it out of the one it is in.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayPlanPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(
        default,
        deserialize_with = "super::format::nullable::deserialize",
        skip_serializing_if = "Option::is_none"
    )]
    pub category_id: Option<Option<Uuid>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<DayPlanItem>>,
}

/// The daily targets an update changes; a figure left out keeps its value.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetsPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_kcal: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protein_g: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fat_g: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carbohydrates_g: Option<f64>,
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
            Self::CreateProduct { .. } => OperationKind::CreateProduct,
            Self::UpdateProduct { .. } => OperationKind::UpdateProduct,
            Self::CreateMeal { .. } => OperationKind::CreateMeal,
            Self::UpdateMeal { .. } => OperationKind::UpdateMeal,
            Self::CreateDayPlan { .. } => OperationKind::CreateDayPlan,
            Self::UpdateDayPlan { .. } => OperationKind::UpdateDayPlan,
            Self::SyncProduct { .. } => OperationKind::SyncProduct,
            Self::UpdateTargets { .. } => OperationKind::UpdateTargets,
        }
    }

    /// The dates this operation acts on, or `None` for an entity edit, which names none.
    pub fn dates(&self) -> Option<DateRange> {
        match self {
            Self::AdjustProductAmount { dates, .. }
            | Self::AddItem { dates, .. }
            | Self::RemoveItem { dates, .. }
            | Self::SwapMeal { dates, .. }
            | Self::AssignPlan { dates, .. }
            | Self::ClearPlan { dates } => Some(*dates),
            Self::CreateProduct { .. }
            | Self::UpdateProduct { .. }
            | Self::CreateMeal { .. }
            | Self::UpdateMeal { .. }
            | Self::CreateDayPlan { .. }
            | Self::UpdateDayPlan { .. }
            | Self::SyncProduct { .. }
            | Self::UpdateTargets { .. } => None,
        }
    }

    /// The kind of entity an entity edit is about, or `None` for a day edit.
    pub fn entity(&self) -> Option<EntityKind> {
        match self {
            Self::CreateProduct { .. } | Self::UpdateProduct { .. } | Self::SyncProduct { .. } => {
                Some(EntityKind::Product)
            }
            Self::CreateMeal { .. } | Self::UpdateMeal { .. } => Some(EntityKind::Meal),
            Self::CreateDayPlan { .. } | Self::UpdateDayPlan { .. } => Some(EntityKind::DayPlan),
            Self::UpdateTargets { .. } => Some(EntityKind::Targets),
            Self::AdjustProductAmount { .. }
            | Self::AddItem { .. }
            | Self::RemoveItem { .. }
            | Self::SwapMeal { .. }
            | Self::AssignPlan { .. }
            | Self::ClearPlan { .. } => None,
        }
    }
}

/// What an entity edit is about, in the order entity edits are applied: a meal can name a
/// product the same proposal creates, and a day plan a meal. The targets name nothing.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntityKind {
    Product,
    Meal,
    DayPlan,
    Targets,
}

/// Whether an entity edit makes something new or changes what is there.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntityAction {
    Create,
    Update,
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
    CreateProduct,
    UpdateProduct,
    CreateMeal,
    UpdateMeal,
    CreateDayPlan,
    UpdateDayPlan,
    SyncProduct,
    UpdateTargets,
}

impl OperationKind {
    /// Every kind of operation there is.
    pub const ALL: [Self; 14] = [
        Self::AdjustProductAmount,
        Self::AddItem,
        Self::RemoveItem,
        Self::SwapMeal,
        Self::AssignPlan,
        Self::ClearPlan,
        Self::CreateProduct,
        Self::UpdateProduct,
        Self::CreateMeal,
        Self::UpdateMeal,
        Self::CreateDayPlan,
        Self::UpdateDayPlan,
        Self::SyncProduct,
        Self::UpdateTargets,
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
            Self::CreateProduct => "create-product",
            Self::UpdateProduct => "update-product",
            Self::CreateMeal => "create-meal",
            Self::UpdateMeal => "update-meal",
            Self::CreateDayPlan => "create-day-plan",
            Self::UpdateDayPlan => "update-day-plan",
            Self::SyncProduct => "sync-product",
            Self::UpdateTargets => "update-targets",
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
/// The days around today with what is on them and what that adds up to, and what could be
/// put on them with its figures, so the model can both turn a sentence into operations
/// over these ids and answer a question about the plan's nutrients. No prices, no history,
/// no weigh-ins.
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
    /// The categories meals and day plans can be filed under.
    #[serde(default)]
    pub categories: Vec<ContextCategory>,
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
    /// The day's nutrients beyond the macros, summed over the products that state each
    /// one. A product that does not state a nutrient adds nothing to it, so a total is a
    /// floor.
    #[serde(default, skip_serializing_if = "Nutrients::is_empty")]
    pub nutrients: Nutrients,
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

/// A day plan the assistant may put on a date, or change.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPlan {
    pub id: Uuid,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    /// What the plan holds, so an update can say what it becomes.
    pub items: Vec<DayPlanItem>,
}

/// A meal the assistant may add to a day, swap in, or change.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextMeal {
    pub id: Uuid,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Uuid>,
    /// What the meal is made of, so an update can say what it becomes.
    pub servings: Vec<Serving>,
}

/// A product the assistant may add to a day, adjust, or change.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextProduct {
    pub id: Uuid,
    pub name: String,
    pub unit: Unit,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    /// Per 100 g or ml, so a new meal can be sized against the targets.
    pub macros: Macros,
    /// Per 100 g or ml, in grams, keyed by the catalogue's names.
    #[serde(default, skip_serializing_if = "Nutrients::is_empty")]
    pub nutrients: Nutrients,
    /// Whether the product came from the catalogue, and so can be synced from it without
    /// being given a code.
    #[serde(default)]
    pub synced: bool,
}

/// A category a meal or day plan may be filed under.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextCategory {
    pub id: Uuid,
    pub name: String,
    pub scope: CategoryScope,
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
    /// One entry per product, meal or day plan the proposal creates or changes, in the
    /// order they would be written.
    #[serde(default)]
    pub entities: Vec<PreviewEntity>,
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

/// What one product, meal or day plan would become.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewEntity {
    pub kind: EntityKind,
    pub action: EntityAction,
    /// Its name or label as it would end up.
    pub label: String,
    /// What changes, in plain words, one line per change.
    pub changes: Vec<String>,
    /// Per 100 g or ml for a product; the whole meal or day for the others.
    pub macros: Macros,
    /// How many dates the assistant can see already use it, and so follow an update.
    pub dates_using: usize,
}

/// What an applied request did to one product, meal, day plan or the targets.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedEntityChange {
    pub kind: EntityKind,
    pub action: EntityAction,
    /// The real id, as the store assigned it for a create; nil for the targets, of which
    /// there is only one set.
    pub id: Uuid,
    pub label: String,
    pub changes: Vec<String>,
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
    /// The products, meals and day plans it created or changed. Empty in records from
    /// before the assistant could.
    #[serde(default)]
    pub entity_changes: Vec<AppliedEntityChange>,
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
    pub entity_changes: Vec<AppliedEntityChange>,
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
