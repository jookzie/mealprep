//! The rules over a plain-language correction to the plan.
//!
//! Four steps, in this order and never fewer: load the slice of data a request could be
//! about, ask the assistant what the request means, check every operation it proposed
//! against that data, and hand the user a [`Preview`]. Applying is a separate call with
//! the operations the user confirmed, re-checked from scratch, because a confirmation
//! comes back over the IPC bridge and is no more trustworthy than the first answer.
//!
//! Checking lowers the operations onto [`DayChange`], one per date. Everything after that
//! — the preview, the writes, the log — is written against `DayChange` and knows nothing
//! about which operation produced it, which is what keeps a new operation down to one
//! enum variant, one arm of [`edit`] and one tool schema in the adapter.

use std::collections::{BTreeMap, HashMap};

use time::{Date, Duration, OffsetDateTime};
use uuid::Uuid;

use super::{
    Mealprep,
    derive::{self, MealIndex, ProductIndex},
    validate,
};
use crate::{
    Entity, Error, Result,
    domain::{
        AppliedChange, AssistantContext, AssistantKeyStatus, AssistantRecord, AssistantRecordDraft,
        ContextDay, ContextItem, ContextMeal, ContextPlan, ContextProduct, DayPlan, DayPlanDraft,
        DayPlanItem, ItemRef, Macros, Operation, Preview, PreviewDay, Product, Proposal, Serving,
        format_iso_date,
    },
    store::{
        Assistant, AssistantStore, CalendarStore, DayPlanStore, MealStore, ProductStore,
        TargetsStore,
    },
};

/// How far back the assistant can see, and therefore how far back it can be asked to
/// change anything. A week is enough to fix the day before yesterday and no more.
const DAYS_BACK: i64 = 7;

/// How far ahead it can see: the four weeks the calendar shows, and the week after.
const DAYS_AHEAD: i64 = 35;

/// The most dates one operation may name. A correction spans days, not seasons, and a
/// bound keeps a misread "every Monday" from rewriting a year.
const MAX_DATES: usize = 35;

/// How many applied requests the log hands back.
const LOG_LIMIT: u32 = 50;

// the resolved shape everything downstream is written against

/// What one date becomes once every operation naming it has been checked.
///
/// The preview reads it, the applier writes it and the log records it, none of them
/// knowing which operation produced it.
#[derive(Clone, Debug, PartialEq)]
pub struct DayChange {
    pub date: Date,
    /// The plan the date ends up holding, or `None` when it ends up unplanned.
    pub plan: Option<PlannedChange>,
    /// The plan the date pointed at before, kept even when that plan has been deleted,
    /// because putting the date back is a matter of re-assigning this id.
    pub previous_day_plan_id: Option<Uuid>,
    pub previous_label: Option<String>,
    /// What changed, one line per operation that touched this date, in order.
    pub changes: Vec<String>,
}

/// The content a date ends up with, and which row it has to be written to.
#[derive(Clone, Debug, PartialEq)]
pub struct PlannedChange {
    /// The plan row the date points at as of this change.
    pub current_plan_id: Uuid,
    /// The plan the content came from: `current_plan_id`, or what that is a copy of.
    pub source_plan_id: Uuid,
    /// Whether `current_plan_id` is already this date's own copy.
    pub is_copy: bool,
    pub label: String,
    pub category_id: Option<Uuid>,
    pub items: Vec<DayPlanItem>,
    /// Set once an operation has changed the items, which is what makes the date need a
    /// copy of its own.
    pub edited: bool,
}

impl DayChange {
    /// Whether applying this change detaches the date onto a new copy of its plan.
    ///
    /// Copy-on-write, once per date: an edited date that is still sharing a plan gets its
    /// own copy, and a date that already has one is edited in place.
    pub fn detaches(&self) -> bool {
        self.plan
            .as_ref()
            .is_some_and(|plan| plan.edited && !plan.is_copy)
    }

    fn planned_mut(&mut self) -> Result<&mut PlannedChange> {
        let date = self.date;
        self.plan
            .as_mut()
            .ok_or_else(|| invalid(format!("{} has no plan to change", format_iso_date(date))))
    }
}

impl PlannedChange {
    fn of(plan: &DayPlan) -> Self {
        Self {
            current_plan_id: plan.id,
            source_plan_id: plan.source_plan_id.unwrap_or(plan.id),
            is_copy: plan.source_plan_id.is_some(),
            label: plan.label.clone(),
            category_id: plan.category_id,
            items: plan.items.clone(),
            edited: false,
        }
    }

    fn draft(&self) -> DayPlanDraft {
        DayPlanDraft {
            label: self.label.clone(),
            category_id: self.category_id,
            items: self.items.clone(),
        }
    }
}

// the data a request is checked against

/// The slice of the user's data a request could be about, loaded once per call.
struct World {
    today: Date,
    targets: Option<Macros>,
    /// What each date in the window is assigned, including a plan since deleted.
    calendar: BTreeMap<Date, Uuid>,
    /// Every plan in play: the ones the user keeps, and the day-local copies the
    /// window's dates point at.
    plans: HashMap<Uuid, DayPlan>,
    /// The plans that can be put on a date, in list order. A copy is never one of them.
    assignable: Vec<ContextPlan>,
    meal_list: Vec<ContextMeal>,
    product_list: Vec<ContextProduct>,
    meals: MealIndex,
    products: ProductIndex,
}

impl World {
    fn window(&self) -> (Date, Date) {
        (
            self.today - Duration::days(DAYS_BACK),
            self.today + Duration::days(DAYS_AHEAD),
        )
    }

    /// The plan on the given date, or `None` when it is unplanned or its plan is gone.
    fn plan_on(&self, date: Date) -> Option<&DayPlan> {
        self.plans.get(self.calendar.get(&date)?)
    }

    /// What the given order adds up to, by the same rules a day plan's own total follows:
    /// a deleted meal drops out, a deleted product counts as nothing but stays an entry.
    fn macros_of(&self, items: &[DayPlanItem]) -> Macros {
        items
            .iter()
            .map(|item| match item {
                DayPlanItem::Meal { meal_id } => self
                    .meals
                    .get(meal_id)
                    .map_or(Macros::ZERO, |meal| meal.macros),
                DayPlanItem::Product { product_id, amount } => derive::serving_macros(
                    &Serving {
                        product_id: *product_id,
                        amount: *amount,
                    },
                    &self.products,
                ),
            })
            .sum()
    }

    /// What the context tells the model, which is everything it is allowed to name.
    fn context(&self) -> AssistantContext {
        let (from, to) = self.window();
        let days = std::iter::successors(Some(from), |date| date.next_day())
            .take_while(|date| *date <= to)
            .map(|date| {
                let plan = self.plan_on(date);
                ContextDay {
                    date,
                    label: plan.map(|plan| plan.label.clone()),
                    items: plan
                        .map(|plan| self.context_items(&plan.items))
                        .unwrap_or_default(),
                    macros: plan.map_or(Macros::ZERO, |plan| self.macros_of(&plan.items)),
                }
            })
            .collect();

        AssistantContext {
            today: self.today,
            targets: self.targets,
            days,
            plans: self.assignable.clone(),
            meals: self.meal_list.clone(),
            products: self.product_list.clone(),
        }
    }

    /// The order as the model sees it: ids with the names they go by, so a request about
    /// "the whey" can be resolved to the product it means.
    ///
    /// An item whose entity has been deleted is left out, because the model cannot be
    /// asked to reason about something with no name.
    fn context_items(&self, items: &[DayPlanItem]) -> Vec<ContextItem> {
        items
            .iter()
            .filter_map(|item| match item {
                DayPlanItem::Meal { meal_id } => {
                    self.meals.get(meal_id).map(|meal| ContextItem::Meal {
                        meal_id: *meal_id,
                        label: meal.entity.label.clone(),
                    })
                }
                DayPlanItem::Product { product_id, amount } => {
                    self.products
                        .get(product_id)
                        .map(|product| ContextItem::Product {
                            product_id: *product_id,
                            name: product.name.clone(),
                            unit: product.unit,
                            amount: *amount,
                        })
                }
            })
            .collect()
    }

    fn assignable_plan(&self, id: Uuid) -> Result<&DayPlan> {
        match self.plans.get(&id) {
            Some(plan) if plan.source_plan_id.is_none() => Ok(plan),
            Some(_) => Err(invalid(format!(
                "day plan {id} is one date's own copy and cannot be assigned"
            ))),
            None => Err(invalid(format!("day plan {id} does not exist"))),
        }
    }

    fn product(&self, id: Uuid) -> Result<&Product> {
        self.products
            .get(&id)
            .ok_or_else(|| invalid(format!("product {id} does not exist")))
    }

    fn meal_label(&self, id: Uuid) -> Result<&str> {
        self.meals
            .get(&id)
            .map(|meal| meal.entity.label.as_str())
            .ok_or_else(|| invalid(format!("meal {id} does not exist")))
    }
}

// the rules

impl<S, C, A> Mealprep<S, C, A>
where
    S: AssistantStore + CalendarStore + DayPlanStore + MealStore + ProductStore + TargetsStore,
    A: Assistant,
{
    /// Asks the assistant what the request means and checks the answer against the real
    /// data, writing nothing.
    ///
    /// The preview it returns is what the user confirms; the operations inside it are
    /// what [`Self::apply_assistant_edit`] takes back.
    pub async fn propose_assistant_edit(&self, request: String) -> Result<Preview> {
        let request = validate::assistant_request(request)?;
        let key = self.assistant_key().await?;
        let world = self.load_world().await?;

        let answer = self
            .assistant
            .propose(&key, &request, &world.context())
            .await?;
        // The request is ours, not the assistant's: a proposal that reworded it would put
        // a sentence the user never typed above the confirmation and into the log.
        let proposal = Proposal {
            request,
            rationale: answer.rationale,
            operations: answer.operations,
        };

        let changes = resolve(&world, &proposal.request, &proposal.operations)?;
        Ok(preview(&world, proposal, &changes))
    }
}

impl<S, C, A> Mealprep<S, C, A>
where
    S: AssistantStore + CalendarStore + DayPlanStore + MealStore + ProductStore + TargetsStore,
{
    /// Applies the operations the user confirmed and records what it did.
    ///
    /// Everything is checked again before anything is written, so a proposal that no
    /// longer fits the data — a product deleted since, a date now in the past — fails
    /// without having touched a row.
    pub async fn apply_assistant_edit(&self, proposal: Proposal) -> Result<AssistantRecord> {
        let request = validate::assistant_request(proposal.request)?;
        let world = self.load_world().await?;
        let changes = resolve(&world, &request, &proposal.operations)?;
        if changes.is_empty() {
            return Err(invalid("there is nothing to apply".to_owned()));
        }

        let mut applied = Vec::with_capacity(changes.len());
        for change in &changes {
            applied.push(self.write_day(change).await?);
        }

        let draft = AssistantRecordDraft {
            request,
            rationale: proposal.rationale,
            operations: proposal.operations,
            changes: applied,
        };
        self.store.record_assistant_request(&draft).await
    }

    /// Writes one date, detaching it onto its own copy of its plan when the edit needs it.
    async fn write_day(&self, change: &DayChange) -> Result<AppliedChange> {
        let (day_plan_id, label) = match &change.plan {
            // Nothing is deleted: the date stops pointing at the plan, the plan stays.
            None => {
                self.store.unassign_calendar_day(change.date).await?;
                (None, None)
            }
            // Only the assignment changed, so the plan itself is left alone.
            Some(plan) if !plan.edited => {
                self.store
                    .assign_calendar_day(change.date, plan.current_plan_id)
                    .await?;
                (Some(plan.current_plan_id), Some(plan.label.clone()))
            }
            // The date already has its own copy, so editing it moves nothing else.
            Some(plan) if plan.is_copy => {
                let updated = self
                    .store
                    .update_day_plan(plan.current_plan_id, &plan.draft())
                    .await?;
                (Some(updated.id), Some(updated.label))
            }
            // Copy-on-write: the date gets its own copy and points at that, so every
            // other date on the original — every past date included — is untouched.
            Some(plan) => {
                let copy = self
                    .store
                    .create_day_plan_copy(plan.source_plan_id, &plan.draft())
                    .await?;
                self.store.assign_calendar_day(change.date, copy.id).await?;
                (Some(copy.id), Some(copy.label))
            }
        };

        Ok(AppliedChange {
            date: change.date,
            previous_day_plan_id: change.previous_day_plan_id,
            previous_label: change.previous_label.clone(),
            day_plan_id,
            label,
            detached: change.detaches(),
            changes: change.changes.clone(),
        })
    }

    async fn load_world(&self) -> Result<World> {
        let today = OffsetDateTime::now_utc().date();
        let from = today - Duration::days(DAYS_BACK);
        let to = today + Duration::days(DAYS_AHEAD);

        let days = self.store.list_calendar_days(from, to).await?;
        let kept = self.store.list_day_plans().await?;
        // The dates in the window may point at day-local copies, which the plan list
        // leaves out, and at plans since deleted, which nothing returns.
        let missing: Vec<Uuid> = days
            .iter()
            .map(|day| day.day_plan_id)
            .filter(|id| kept.iter().all(|plan| plan.id != *id))
            .collect();
        let copies = self.store.list_day_plans_by_ids(&missing).await?;

        let assignable = kept
            .iter()
            .map(|plan| ContextPlan {
                id: plan.id,
                label: plan.label.clone(),
            })
            .collect();
        let plans: HashMap<Uuid, DayPlan> = kept
            .into_iter()
            .chain(copies)
            .map(|plan| (plan.id, plan))
            .collect();

        let meals = self.store.list_meals().await?;
        let meal_list = meals
            .iter()
            .map(|meal| ContextMeal {
                id: meal.id,
                label: meal.label.clone(),
            })
            .collect();
        let products = self.store.list_products().await?;
        let product_list = products
            .iter()
            .map(|product| ContextProduct {
                id: product.id,
                name: product.name.clone(),
                unit: product.unit,
            })
            .collect();

        let products = derive::index_products(products);
        let meals: MealIndex = meals
            .into_iter()
            .map(|meal| (meal.id, derive::derive_meal(meal, &products)))
            .collect();

        Ok(World {
            today,
            targets: self
                .store
                .get_targets()
                .await?
                .map(|targets| targets.macros),
            calendar: days
                .into_iter()
                .map(|day| (day.date, day.day_plan_id))
                .collect(),
            plans,
            assignable,
            meal_list,
            product_list,
            meals,
            products,
        })
    }
}

impl<S, C, A> Mealprep<S, C, A>
where
    S: AssistantStore,
{
    /// Reports whether the assistant has a key, and the last four characters of it.
    pub async fn assistant_key_status(&self) -> Result<AssistantKeyStatus> {
        let key = self.store.get_assistant_key().await?;
        Ok(key_status(key.as_deref()))
    }

    /// Stores the key the user pasted in. It stays on the device.
    pub async fn set_assistant_key(&self, key: String) -> Result<AssistantKeyStatus> {
        let key = validate::assistant_key(key)?;
        self.store.set_assistant_key(&key).await?;
        Ok(key_status(Some(&key)))
    }

    /// Forgets the stored key, which turns the assistant off.
    pub async fn clear_assistant_key(&self) -> Result<()> {
        self.store.clear_assistant_key().await
    }

    /// Lists what the assistant has been asked to do and what it changed, newest first.
    pub async fn list_assistant_requests(&self) -> Result<Vec<AssistantRecord>> {
        self.store.list_assistant_requests(LOG_LIMIT).await
    }

    async fn assistant_key(&self) -> Result<String> {
        self.store.get_assistant_key().await?.ok_or_else(|| {
            invalid("the assistant has no API key yet; paste one in to turn it on".to_owned())
        })
    }
}

/// The last four characters, which is as much of a key as anything outside the store sees.
fn key_status(key: Option<&str>) -> AssistantKeyStatus {
    let Some(key) = key else {
        return AssistantKeyStatus::default();
    };
    let hint: String = key
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    AssistantKeyStatus {
        present: true,
        hint: Some(hint),
    }
}

/// Checks every operation against the data and says what each date it names becomes.
///
/// Operations are checked in the order they were proposed and compose: two corrections to
/// the same day both land, rather than the second overwriting the first's result.
fn resolve(world: &World, request: &str, operations: &[Operation]) -> Result<Vec<DayChange>> {
    let past_allowed = validate::past_dates_requested(request, world.today);
    let mut changes: BTreeMap<Date, DayChange> = BTreeMap::new();

    for operation in operations {
        for date in checked_dates(world, operation, past_allowed)? {
            let change = changes
                .entry(date)
                .or_insert_with(|| initial_change(world, date));
            edit(world, operation, change)?;
        }
    }

    // A date an operation named but did not change is not a change to confirm.
    let changes = changes
        .into_values()
        .filter(|change| !change.changes.is_empty())
        .collect();
    Ok(changes)
}

/// The dates an operation may act on, or why it may not act on them.
///
/// Three bounds, each its own refusal rather than a silent trim: the range has to run
/// forwards and stay short, it has to lie inside the window the assistant can see, and a
/// date before today needs the request to have asked for it in so many words.
fn checked_dates(world: &World, operation: &Operation, past_allowed: bool) -> Result<Vec<Date>> {
    let range = operation.dates();
    let dates: Vec<Date> = range.dates().collect();
    let kind = operation.kind().name();

    if dates.is_empty() {
        return Err(invalid(format!(
            "{kind} ends on {} before it starts on {}",
            format_iso_date(range.to),
            format_iso_date(range.from)
        )));
    }
    if dates.len() > MAX_DATES {
        return Err(invalid(format!(
            "{kind} covers {} days, more than the {MAX_DATES} one correction may",
            dates.len()
        )));
    }

    let (from, to) = world.window();
    for date in &dates {
        if *date < from || *date > to {
            return Err(invalid(format!(
                "{kind} names {}, outside the {} to {} the assistant can see",
                format_iso_date(*date),
                format_iso_date(from),
                format_iso_date(to)
            )));
        }
        if *date < world.today && !past_allowed {
            return Err(invalid(format!(
                "{kind} names {}, which has passed; say the past date in the request if that is what you mean",
                format_iso_date(*date)
            )));
        }
    }
    Ok(dates)
}

fn initial_change(world: &World, date: Date) -> DayChange {
    let assigned = world.calendar.get(&date).copied();
    let plan = world.plan_on(date);
    DayChange {
        date,
        plan: plan.map(PlannedChange::of),
        previous_day_plan_id: assigned,
        previous_label: plan.map(|plan| plan.label.clone()),
        changes: Vec::new(),
    }
}

/// What one operation does to one date.
///
/// This is the only place an operation's meaning is written down. A new variant of
/// [`Operation`] needs an arm here and a tool schema in the adapter, and nothing else:
/// everything downstream reads the [`DayChange`] this produced.
fn edit(world: &World, operation: &Operation, change: &mut DayChange) -> Result<()> {
    let line = match operation {
        Operation::ClearPlan { .. } => {
            let label = change
                .plan
                .as_ref()
                .map(|plan| plan.label.clone())
                .ok_or_else(|| {
                    invalid(format!(
                        "{} has no plan to clear",
                        format_iso_date(change.date)
                    ))
                })?;
            change.plan = None;
            format!("{label} removed, leaving the day unplanned")
        }

        Operation::AssignPlan { day_plan_id, .. } => {
            let plan = world.assignable_plan(*day_plan_id)?;
            let line = match &change.plan {
                Some(current) if current.current_plan_id == plan.id => {
                    format!("{} already", plan.label)
                }
                Some(current) => format!("{} instead of {}", plan.label, current.label),
                None => format!("{} on a day that had nothing", plan.label),
            };
            change.plan = Some(PlannedChange::of(plan));
            line
        }

        Operation::AdjustProductAmount {
            product_id,
            delta_amount,
            ..
        } => {
            let product = world.product(*product_id)?;
            let (name, unit) = (product.name.clone(), product.unit);
            let date = change.date;
            let plan = change.planned_mut()?;

            let wanted = ItemRef::Product {
                product_id: *product_id,
            };
            let position = sole_position(&plan.items, &wanted, &name, date)?.ok_or_else(|| {
                invalid(format!(
                    "{name} is not an item of its own on {}; a product inside a meal cannot be adjusted for one day",
                    format_iso_date(date)
                ))
            })?;
            // The reference named a product, so the entry it matched holds one.
            let DayPlanItem::Product { amount, .. } = &mut plan.items[position] else {
                return Err(invalid(format!(
                    "{name} is not a product on {}",
                    format_iso_date(date)
                )));
            };

            let was = *amount;
            let now = was + delta_amount;
            if !now.is_finite() || now <= 0.0 {
                return Err(invalid(format!(
                    "{was} {unit} of {name} on {} cannot change by {delta_amount}; remove the item instead",
                    format_iso_date(date)
                )));
            }
            *amount = now;
            plan.edited = true;
            format!("{name} {was} {unit} → {now} {unit}")
        }

        Operation::AddItem { item, .. } => {
            let line = match item {
                DayPlanItem::Meal { meal_id } => format!("{} added", world.meal_label(*meal_id)?),
                DayPlanItem::Product { product_id, amount } => {
                    let product = world.product(*product_id)?;
                    if !amount.is_finite() || *amount <= 0.0 {
                        return Err(invalid(format!(
                            "{amount} is not an amount of {}",
                            product.name
                        )));
                    }
                    format!("{amount} {} of {} added", product.unit, product.name)
                }
            };
            let plan = change.planned_mut()?;
            plan.items.push(*item);
            plan.edited = true;
            line
        }

        Operation::RemoveItem { item, .. } => {
            let name = match item {
                ItemRef::Meal { meal_id } => world.meal_label(*meal_id)?.to_owned(),
                ItemRef::Product { product_id } => world.product(*product_id)?.name.clone(),
            };
            let line = format!("{name} removed");
            let date = change.date;
            let plan = change.planned_mut()?;
            let position = sole_position(&plan.items, item, &name, date)?.ok_or_else(|| {
                invalid(format!(
                    "{} does not hold that item on {}",
                    plan.label,
                    format_iso_date(date)
                ))
            })?;
            plan.items.remove(position);
            plan.edited = true;
            line
        }

        Operation::SwapMeal {
            from_meal_id,
            to_meal_id,
            ..
        } => {
            let from = world.meal_label(*from_meal_id)?.to_owned();
            let to = world.meal_label(*to_meal_id)?.to_owned();
            let date = change.date;
            let plan = change.planned_mut()?;
            let wanted = ItemRef::Meal {
                meal_id: *from_meal_id,
            };
            let position = sole_position(&plan.items, &wanted, &from, date)?.ok_or_else(|| {
                invalid(format!(
                    "{from} is not on {}, so there is nothing to swap",
                    format_iso_date(date)
                ))
            })?;
            plan.items[position] = DayPlanItem::Meal {
                meal_id: *to_meal_id,
            };
            plan.edited = true;
            format!("{to} in place of {from}")
        }
    };

    change.changes.push(line);
    Ok(())
}

/// Where in a day's order the reference points, when exactly one entry names it.
///
/// `None` says nothing names it, which each caller puts in its own words. Two entries is
/// refused here: an item is addressed by id, and a day can legitimately hold the same one
/// twice — whey loose at breakfast and again after training. No operation can tell those
/// apart, because order is the only sequence a day plan carries, so taking the first would
/// move an item the user did not name and show a change line that does not say which.
fn sole_position(
    items: &[DayPlanItem],
    wanted: &ItemRef,
    name: &str,
    date: Date,
) -> Result<Option<usize>> {
    let mut found = items
        .iter()
        .enumerate()
        .filter(|(_, item)| names(item, wanted))
        .map(|(position, _)| position);
    let Some(position) = found.next() else {
        return Ok(None);
    };
    if found.next().is_some() {
        return Err(invalid(format!(
            "{name} is on {} more than once, so naming one of them is ambiguous; change that day's plan by hand",
            format_iso_date(date)
        )));
    }
    Ok(Some(position))
}

/// Whether the given entry of a day's order is the one the reference names.
fn names(item: &DayPlanItem, wanted: &ItemRef) -> bool {
    match (item, wanted) {
        (DayPlanItem::Meal { meal_id }, ItemRef::Meal { meal_id: wanted }) => meal_id == wanted,
        (DayPlanItem::Product { product_id, .. }, ItemRef::Product { product_id: wanted }) => {
            product_id == wanted
        }
        _ => false,
    }
}

/// What the user reads before confirming: the dates that change, what changes on each,
/// and where that leaves the day against the targets.
fn preview(world: &World, proposal: Proposal, changes: &[DayChange]) -> Preview {
    let days = changes
        .iter()
        .map(|change| {
            let before = world
                .plan_on(change.date)
                .map_or(Macros::ZERO, |plan| world.macros_of(&plan.items));
            let after = change
                .plan
                .as_ref()
                .map_or(Macros::ZERO, |plan| world.macros_of(&plan.items));
            PreviewDay {
                date: change.date,
                label: change.plan.as_ref().map(|plan| plan.label.clone()),
                changes: change.changes.clone(),
                before,
                after,
                // Against the target rather than against `before`: the target is the
                // figure the day is planned towards, and the deviation is what to read.
                target_delta: world.targets.map(|target| after + target.scale(-1.0)),
                detaches: change.detaches(),
            }
        })
        .collect();

    Preview {
        proposal,
        days,
        targets: world.targets,
    }
}

fn invalid(reason: String) -> Error {
    Error::Invalid {
        entity: Entity::AssistantRequest,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;
    use crate::domain::{DateRange, Meal, Nutrients, Unit};

    const TODAY: Date = date!(2026 - 10 - 20);

    fn product(name: &str, energy_kcal: f64) -> Product {
        Product {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            unit: Unit::Gram,
            macros: Macros {
                energy_kcal,
                fat_g: 2.0,
                protein_g: 20.0,
                carbohydrates_g: 10.0,
            },
            nutrients: Nutrients::new(),
            cost: Some(1.0),
            brand: None,
            source_code: None,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }

    fn meal(label: &str, servings: Vec<Serving>) -> Meal {
        Meal {
            id: Uuid::new_v4(),
            label: label.to_owned(),
            category_id: None,
            servings,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }

    fn day_plan(label: &str, items: Vec<DayPlanItem>, source_plan_id: Option<Uuid>) -> DayPlan {
        DayPlan {
            id: Uuid::new_v4(),
            label: label.to_owned(),
            category_id: None,
            source_plan_id,
            items,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
        }
    }

    /// Four days on one shared plan, two of them already past: the shape the whole
    /// copy-on-write question is about.
    struct Fixture {
        world: World,
        whey: Uuid,
        banana: Uuid,
        porridge: Uuid,
        omelette: Uuid,
        training: Uuid,
        rest: Uuid,
    }

    fn fixture() -> Fixture {
        let whey = product("Whey", 380.0);
        let banana = product("Banana", 89.0);
        let porridge = meal(
            "Porridge",
            vec![Serving {
                product_id: banana.id,
                amount: 100.0,
            }],
        );
        let omelette = meal("Omelette", Vec::new());

        let training = day_plan(
            "Training day",
            vec![
                DayPlanItem::Meal {
                    meal_id: porridge.id,
                },
                DayPlanItem::Product {
                    product_id: whey.id,
                    amount: 60.0,
                },
            ],
            None,
        );
        let rest = day_plan("Rest day", Vec::new(), None);

        // Three days back and four ahead, every one of them on the training plan.
        let calendar =
            std::iter::successors(Some(TODAY - Duration::days(3)), |date| date.next_day())
                .take(8)
                .map(|date| (date, training.id))
                .collect();

        let products = derive::index_products(vec![whey.clone(), banana.clone()]);
        let meals: MealIndex = [porridge.clone(), omelette.clone()]
            .into_iter()
            .map(|meal| (meal.id, derive::derive_meal(meal, &products)))
            .collect();

        let world = World {
            today: TODAY,
            targets: Some(Macros {
                energy_kcal: 2000.0,
                fat_g: 65.0,
                protein_g: 140.0,
                carbohydrates_g: 220.0,
            }),
            calendar,
            assignable: [&training, &rest]
                .into_iter()
                .map(|plan| ContextPlan {
                    id: plan.id,
                    label: plan.label.clone(),
                })
                .collect(),
            meal_list: [&porridge, &omelette]
                .into_iter()
                .map(|meal| ContextMeal {
                    id: meal.id,
                    label: meal.label.clone(),
                })
                .collect(),
            product_list: [&whey, &banana]
                .into_iter()
                .map(|product| ContextProduct {
                    id: product.id,
                    name: product.name.clone(),
                    unit: product.unit,
                })
                .collect(),
            plans: [training.clone(), rest.clone()]
                .into_iter()
                .map(|plan| (plan.id, plan))
                .collect(),
            meals,
            products,
        };

        Fixture {
            world,
            whey: whey.id,
            banana: banana.id,
            porridge: porridge.id,
            omelette: omelette.id,
            training: training.id,
            rest: rest.id,
        }
    }

    fn next_days(days: i64) -> DateRange {
        DateRange {
            from: TODAY,
            to: TODAY + Duration::days(days - 1),
        }
    }

    #[test]
    fn adjusts_the_named_days_and_leaves_the_rest_to_copy_on_write() {
        let f = fixture();
        let operations = [Operation::AdjustProductAmount {
            dates: next_days(4),
            product_id: f.whey,
            delta_amount: -40.0,
        }];

        let changes = resolve(&f.world, "next 4 days I have 40 g less whey", &operations).unwrap();
        assert_eq!(changes.len(), 4);
        for change in &changes {
            assert!(change.date >= TODAY, "{} is in the past", change.date);
            let plan = change.plan.as_ref().unwrap();
            assert_eq!(plan.source_plan_id, f.training);
            assert!(plan.edited);
            // The date is still sharing the plan, so applying detaches it onto a copy.
            assert!(change.detaches());
            assert_eq!(
                plan.items,
                vec![
                    DayPlanItem::Meal {
                        meal_id: f.porridge
                    },
                    DayPlanItem::Product {
                        product_id: f.whey,
                        amount: 20.0,
                    },
                ]
            );
        }

        // The past days share the same plan and are not in the result at all.
        assert!(
            changes
                .iter()
                .all(|change| change.date != TODAY - Duration::days(1))
        );
    }

    #[test]
    fn a_date_that_already_has_its_own_copy_is_edited_in_place() {
        let mut f = fixture();
        let source = f.world.plans[&f.training].clone();
        let copy = day_plan("Training day", source.items.clone(), Some(f.training));
        let copy_id = copy.id;
        f.world.calendar.insert(TODAY, copy_id);
        f.world.plans.insert(copy_id, copy);

        let operations = [Operation::AdjustProductAmount {
            dates: DateRange::just(TODAY),
            product_id: f.whey,
            delta_amount: -40.0,
        }];
        let changes = resolve(&f.world, "today 40 g less whey", &operations).unwrap();

        let plan = changes[0].plan.as_ref().unwrap();
        assert_eq!(plan.current_plan_id, copy_id);
        assert_eq!(plan.source_plan_id, f.training);
        assert!(plan.is_copy && plan.edited);
        // Nothing to detach: the date is already the only one on this row.
        assert!(!changes[0].detaches());
    }

    #[test]
    fn refuses_a_past_date_unless_the_request_asked_for_one() {
        let f = fixture();
        let operations = [Operation::AdjustProductAmount {
            dates: DateRange::just(TODAY - Duration::days(1)),
            product_id: f.whey,
            delta_amount: -40.0,
        }];

        let Err(Error::Invalid { reason, .. }) = resolve(&f.world, "40 g less whey", &operations)
        else {
            panic!("a past date should be refused");
        };
        assert!(reason.contains("has passed"), "{reason}");

        // The same operation, from a request that says so, goes through.
        let changes = resolve(&f.world, "yesterday I had 40 g less whey", &operations).unwrap();
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn refuses_a_date_outside_the_window_and_a_range_that_runs_backwards() {
        let f = fixture();
        let far = DateRange::just(TODAY + Duration::days(DAYS_AHEAD + 1));
        let backwards = DateRange {
            from: TODAY + Duration::days(2),
            to: TODAY,
        };

        for dates in [far, backwards] {
            let operations = [Operation::ClearPlan { dates }];
            assert!(
                matches!(
                    resolve(&f.world, "clear those days", &operations),
                    Err(Error::Invalid { .. })
                ),
                "{dates:?} should be refused"
            );
        }
    }

    #[test]
    fn refuses_an_unknown_product_an_impossible_amount_and_a_product_inside_a_meal() {
        let f = fixture();

        let unknown = [Operation::AdjustProductAmount {
            dates: DateRange::just(TODAY),
            product_id: Uuid::new_v4(),
            delta_amount: -10.0,
        }];
        assert!(matches!(
            resolve(&f.world, "less of that", &unknown),
            Err(Error::Invalid { .. })
        ));

        // 60 g of whey cannot lose 60: an item is removed, never left at nothing.
        let impossible = [Operation::AdjustProductAmount {
            dates: DateRange::just(TODAY),
            product_id: f.whey,
            delta_amount: -60.0,
        }];
        let Err(Error::Invalid { reason, .. }) = resolve(&f.world, "no whey today", &impossible)
        else {
            panic!("an amount of zero should be refused");
        };
        assert!(reason.contains("remove the item instead"), "{reason}");

        // The banana is inside the porridge, so it is not one day's to adjust.
        let nested = [Operation::AdjustProductAmount {
            dates: DateRange::just(TODAY),
            product_id: f.banana,
            delta_amount: -20.0,
        }];
        let Err(Error::Invalid { reason, .. }) = resolve(&f.world, "less banana", &nested) else {
            panic!("a product inside a meal should be refused");
        };
        assert!(reason.contains("inside a meal"), "{reason}");
    }

    #[test]
    fn refuses_an_item_the_day_holds_twice() {
        let mut f = fixture();
        // Whey loose at breakfast and again after training. Both entries name the same
        // product, and nothing in an operation can say which of them is meant.
        f.world
            .plans
            .get_mut(&f.training)
            .unwrap()
            .items
            .push(DayPlanItem::Product {
                product_id: f.whey,
                amount: 30.0,
            });

        let ambiguous = [
            Operation::AdjustProductAmount {
                dates: DateRange::just(TODAY),
                product_id: f.whey,
                delta_amount: -40.0,
            },
            Operation::RemoveItem {
                dates: DateRange::just(TODAY),
                item: ItemRef::Product {
                    product_id: f.whey,
                },
            },
        ];

        for operation in ambiguous {
            let Err(Error::Invalid { reason, .. }) =
                resolve(&f.world, "40 g less whey", &[operation])
            else {
                panic!("an item the day holds twice should be refused");
            };
            assert!(reason.contains("more than once"), "{reason}");
        }
    }

    #[test]
    fn operations_on_one_date_compose_in_order() {
        let f = fixture();
        let operations = [
            Operation::SwapMeal {
                dates: DateRange::just(TODAY),
                from_meal_id: f.porridge,
                to_meal_id: f.omelette,
            },
            Operation::RemoveItem {
                dates: DateRange::just(TODAY),
                item: ItemRef::Product { product_id: f.whey },
            },
        ];

        let changes = resolve(
            &f.world,
            "omelette instead of porridge, no whey",
            &operations,
        )
        .unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].changes.len(), 2);
        assert_eq!(
            changes[0].plan.as_ref().unwrap().items,
            vec![DayPlanItem::Meal {
                meal_id: f.omelette
            }]
        );
    }

    #[test]
    fn assigning_then_editing_writes_one_copy_of_the_assigned_plan() {
        let f = fixture();
        let operations = [
            Operation::AssignPlan {
                dates: DateRange::just(TODAY + Duration::days(1)),
                day_plan_id: f.rest,
            },
            Operation::AddItem {
                dates: DateRange::just(TODAY + Duration::days(1)),
                item: DayPlanItem::Product {
                    product_id: f.whey,
                    amount: 30.0,
                },
            },
        ];

        let changes = resolve(&f.world, "rest day tomorrow with a shake", &operations).unwrap();
        let plan = changes[0].plan.as_ref().unwrap();
        assert_eq!(plan.source_plan_id, f.rest);
        assert!(plan.edited && !plan.is_copy);
        assert!(changes[0].detaches());
        assert_eq!(changes[0].previous_day_plan_id, Some(f.training));
    }

    #[test]
    fn clearing_a_day_keeps_what_it_pointed_at_for_the_log() {
        let f = fixture();
        let operations = [Operation::ClearPlan {
            dates: DateRange::just(TODAY),
        }];
        let changes = resolve(&f.world, "nothing planned today", &operations).unwrap();

        assert!(changes[0].plan.is_none());
        assert!(!changes[0].detaches());
        assert_eq!(changes[0].previous_day_plan_id, Some(f.training));
        assert_eq!(changes[0].previous_label.as_deref(), Some("Training day"));

        // Clearing a day that has nothing is a refusal, not a no-op.
        let empty = [Operation::ClearPlan {
            dates: DateRange::just(TODAY + Duration::days(30)),
        }];
        assert!(matches!(
            resolve(&f.world, "nothing planned then", &empty),
            Err(Error::Invalid { .. })
        ));
    }

    #[test]
    fn the_preview_counts_the_day_before_and_after_against_the_targets() {
        let f = fixture();
        let operations = [Operation::AdjustProductAmount {
            dates: DateRange::just(TODAY),
            product_id: f.whey,
            delta_amount: -40.0,
        }];
        let changes = resolve(&f.world, "40 g less whey today", &operations).unwrap();
        let proposal = Proposal {
            request: "40 g less whey today".to_owned(),
            rationale: "Dropped the shake".to_owned(),
            operations: operations.to_vec(),
        };

        let preview = preview(&f.world, proposal, &changes);
        assert_eq!(preview.days.len(), 1);
        let day = &preview.days[0];

        // Porridge (100 g of banana) plus 60 g of whey, then plus 20 g of whey.
        assert!((day.before.energy_kcal - (89.0 + 0.6 * 380.0)).abs() < 1e-9);
        assert!((day.after.energy_kcal - (89.0 + 0.2 * 380.0)).abs() < 1e-9);
        assert!((day.after.energy_kcal - day.before.energy_kcal + 152.0).abs() < 1e-9);

        let delta = day.target_delta.unwrap();
        assert!((delta.energy_kcal - (day.after.energy_kcal - 2000.0)).abs() < 1e-9);
        assert_eq!(day.label.as_deref(), Some("Training day"));
        assert!(day.detaches);
    }

    #[test]
    fn the_context_covers_the_whole_window_and_names_what_is_on_it() {
        let f = fixture();
        let context = f.world.context();
        let (from, to) = f.world.window();

        assert_eq!(context.today, TODAY);
        assert_eq!(
            context.days.len(),
            usize::try_from((to - from).whole_days() + 1).unwrap()
        );
        assert_eq!(context.days.first().unwrap().date, from);
        assert_eq!(context.days.last().unwrap().date, to);
        assert_eq!(context.plans.len(), 2);

        let today = context
            .days
            .iter()
            .find(|day| day.date == TODAY)
            .expect("today is in the window");
        assert_eq!(today.label.as_deref(), Some("Training day"));
        assert_eq!(
            today.items,
            vec![
                ContextItem::Meal {
                    meal_id: f.porridge,
                    label: "Porridge".to_owned(),
                },
                ContextItem::Product {
                    product_id: f.whey,
                    name: "Whey".to_owned(),
                    unit: Unit::Gram,
                    amount: 60.0,
                },
            ]
        );

        // A day with nothing on it is still in the context, so a plan can be put on it.
        let far = context.days.last().unwrap();
        assert!(far.label.is_none() && far.items.is_empty());
    }

    #[test]
    fn a_key_is_never_reported_in_full() {
        assert_eq!(key_status(None), AssistantKeyStatus::default());
        let status = key_status(Some("sk-ant-api03-secret-tail"));
        assert!(status.present);
        assert_eq!(status.hint.as_deref(), Some("tail"));
    }
}
