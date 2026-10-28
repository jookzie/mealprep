//! The rules over a plain-language correction to the plan.
//!
//! Four steps, in this order and never fewer: load the slice of data a request could be
//! about, ask the assistant what the request means, check every operation it proposed
//! against that data, and hand the user a [`Preview`]. Applying is a separate call with
//! the operations the user confirmed, re-checked from scratch, because a confirmation
//! comes back over the IPC bridge and is no more trustworthy than the first answer.
//!
//! Checking lowers the operations onto [`DayChange`], one per date, and [`EntityChange`],
//! one per product, meal or day plan created or changed. Everything after that — the
//! preview, the writes, the log — is written against those and knows nothing about which
//! operation produced them, which is what keeps a new operation down to one enum variant,
//! one arm of [`edit`] or [`edit_entity`] and one tool schema in the adapter.
//!
//! Entity edits are checked first, against a copy of the data they then change, so the
//! day edits of the same proposal can name what they create and see what they change.
//!
//! The one edit that needs the outside world is a sync from the catalogue. Its entry is
//! fetched before checking starts, so checking stays a function of the data alone.

use std::collections::{BTreeMap, HashMap, HashSet, hash_map::Entry};

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
        AppliedChange, AppliedEntityChange, AssistantContext, AssistantKeyStatus, AssistantRecord,
        AssistantRecordDraft, CatalogueEntry, Category, CategoryScope, ContextCategory, ContextDay,
        ContextItem, ContextMeal, ContextPlan, ContextProduct, DayPlan, DayPlanDraft, DayPlanItem,
        EntityAction, EntityKind, ItemRef, MacroField, Macros, Meal, MealDraft, Nutrients,
        Operation, Preview, PreviewDay, PreviewEntity, Product, ProductDraft, Proposal, Serving,
        format_iso_date,
    },
    store::{
        Assistant, AssistantStore, CalendarStore, Catalogue, CategoryStore, DayPlanStore,
        MealStore, ProductStore, TargetsStore,
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

/// What one product, meal or day plan becomes once its operation has been checked.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityChange {
    pub action: EntityAction,
    /// The entity's id; for a create, the provisional one the model picked.
    pub id: Uuid,
    /// The whole entity as it is to be written, the fields an update left alone included.
    pub draft: EntityDraft,
    /// What changes, in plain words.
    pub changes: Vec<String>,
    /// The catalogue code a product is synced from from now on, when a sync names one it
    /// was not imported under.
    pub link: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EntityDraft {
    Product(ProductDraft),
    Meal(MealDraft),
    DayPlan(DayPlanDraft),
    Targets(Macros),
}

impl EntityDraft {
    fn kind(&self) -> EntityKind {
        match self {
            Self::Product(_) => EntityKind::Product,
            Self::Meal(_) => EntityKind::Meal,
            Self::DayPlan(_) => EntityKind::DayPlan,
            Self::Targets(_) => EntityKind::Targets,
        }
    }

    fn label(&self) -> &str {
        match self {
            Self::Product(draft) => &draft.name,
            Self::Meal(draft) => &draft.label,
            Self::DayPlan(draft) => &draft.label,
            Self::Targets(_) => "Daily targets",
        }
    }
}

/// Everything a proposal comes to: the entities written first, then the dates, and the
/// data as it stands once the entity edits are in.
struct Resolved {
    entities: Vec<EntityChange>,
    days: Vec<DayChange>,
    after: World,
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
#[derive(Clone)]
struct World {
    today: Date,
    targets: Option<Macros>,
    /// What each date in the window is assigned, including a plan since deleted.
    calendar: BTreeMap<Date, Uuid>,
    /// Every plan in play: the ones the user keeps, and the day-local copies the
    /// window's dates point at.
    plans: HashMap<Uuid, DayPlan>,
    /// The plans that can be put on a date, in list order. A copy is never one of them.
    assignable: Vec<Uuid>,
    /// The meals and products in list order, for the context.
    meal_order: Vec<Uuid>,
    product_order: Vec<Uuid>,
    meals: MealIndex,
    products: ProductIndex,
    /// The categories of both scopes, in list order.
    categories: Vec<Category>,
    /// The catalogue entries the proposal's syncs name, by code, fetched before checking.
    catalogue: HashMap<String, CatalogueEntry>,
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

    /// What the given order adds up to beyond the macros, by the rules of
    /// [`Self::macros_of`]: a deleted meal or product adds nothing.
    fn nutrients_of(&self, items: &[DayPlanItem]) -> Nutrients {
        let mut totals = Nutrients::new();
        for item in items {
            match item {
                DayPlanItem::Meal { meal_id } => {
                    let servings = self
                        .meals
                        .get(meal_id)
                        .map_or(&[][..], |meal| &meal.entity.servings);
                    for serving in servings {
                        derive::add_serving_nutrients(serving, &self.products, &mut totals);
                    }
                }
                DayPlanItem::Product { product_id, amount } => derive::add_serving_nutrients(
                    &Serving {
                        product_id: *product_id,
                        amount: *amount,
                    },
                    &self.products,
                    &mut totals,
                ),
            }
        }
        totals
            .into_iter()
            .map(|(key, value)| (key, significant(value)))
            .collect()
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
                    nutrients: plan
                        .map(|plan| self.nutrients_of(&plan.items))
                        .unwrap_or_default(),
                }
            })
            .collect();

        AssistantContext {
            today: self.today,
            targets: self.targets,
            days,
            plans: self
                .assignable
                .iter()
                .filter_map(|id| self.plans.get(id))
                .map(|plan| ContextPlan {
                    id: plan.id,
                    label: plan.label.clone(),
                    category_id: plan.category_id,
                    items: plan.items.clone(),
                })
                .collect(),
            meals: self
                .meal_order
                .iter()
                .filter_map(|id| self.meals.get(id))
                .map(|meal| ContextMeal {
                    id: meal.entity.id,
                    label: meal.entity.label.clone(),
                    category_id: meal.entity.category_id,
                    servings: meal.entity.servings.clone(),
                })
                .collect(),
            products: self
                .product_order
                .iter()
                .filter_map(|id| self.products.get(id))
                .map(|product| ContextProduct {
                    id: product.id,
                    name: product.name.clone(),
                    unit: product.unit,
                    brand: product.brand.clone(),
                    macros: product.macros,
                    nutrients: product.nutrients.clone(),
                    synced: product.source_code.is_some(),
                })
                .collect(),
            categories: self
                .categories
                .iter()
                .map(|category| ContextCategory {
                    id: category.id,
                    name: category.name.clone(),
                    scope: category.scope,
                })
                .collect(),
        }
    }

    /// Re-derives every meal's totals, after a product they may hold has changed.
    fn rederive_meals(&mut self) {
        let products = &self.products;
        for meal in self.meals.values_mut() {
            *meal = derive::derive_meal(meal.entity.clone(), products);
        }
    }

    /// How many dates in view are on a plan that holds the entity, directly or through a
    /// meal — the dates an update to it reaches.
    fn dates_using(&self, kind: EntityKind, id: Uuid) -> usize {
        self.calendar
            .iter()
            .filter(|(_, plan_id)| match kind {
                EntityKind::DayPlan => **plan_id == id,
                EntityKind::Meal | EntityKind::Product => {
                    self.plans.get(plan_id).is_some_and(|plan| {
                        plan.items.iter().any(|item| self.item_uses(item, kind, id))
                    })
                }
                // Every day is measured against them, so "how many use them" says nothing.
                EntityKind::Targets => false,
            })
            .count()
    }

    fn item_uses(&self, item: &DayPlanItem, kind: EntityKind, id: Uuid) -> bool {
        match (item, kind) {
            (DayPlanItem::Meal { meal_id }, EntityKind::Meal) => *meal_id == id,
            (DayPlanItem::Meal { meal_id }, EntityKind::Product) => {
                self.meals.get(meal_id).is_some_and(|meal| {
                    meal.entity
                        .servings
                        .iter()
                        .any(|serving| serving.product_id == id)
                })
            }
            (DayPlanItem::Product { product_id, .. }, EntityKind::Product) => *product_id == id,
            _ => false,
        }
    }

    /// "80 g Oats, 30 g Whey", for a change line.
    fn servings_text(&self, servings: &[Serving]) -> String {
        servings
            .iter()
            .map(|serving| match self.products.get(&serving.product_id) {
                Some(product) => format!("{} {} {}", serving.amount, product.unit, product.name),
                None => format!("{} of a removed product", serving.amount),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// "Porridge, 30 g Whey", for a change line.
    fn items_text(&self, items: &[DayPlanItem]) -> String {
        items
            .iter()
            .map(|item| match item {
                DayPlanItem::Meal { meal_id } => self.meals.get(meal_id).map_or_else(
                    || "a removed meal".to_owned(),
                    |meal| meal.entity.label.clone(),
                ),
                DayPlanItem::Product { product_id, amount } => self.servings_text(&[Serving {
                    product_id: *product_id,
                    amount: *amount,
                }]),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Fails unless every meal and product the items name exists.
    fn check_items(&self, items: &[DayPlanItem]) -> Result<()> {
        for item in items {
            match item {
                DayPlanItem::Meal { meal_id } => {
                    self.meal_label(*meal_id)?;
                }
                DayPlanItem::Product { product_id, .. } => {
                    self.product(*product_id)?;
                }
            }
        }
        Ok(())
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

    /// The name of the given category, which has to exist and group what is filed in it.
    fn category_name(&self, id: Uuid, scope: CategoryScope) -> Result<&str> {
        match self.categories.iter().find(|category| category.id == id) {
            Some(category) if category.scope == scope => Ok(&category.name),
            Some(category) => Err(invalid(format!(
                "category {} groups {}, not {}",
                category.name,
                category.scope.grouped(),
                scope.grouped()
            ))),
            None => Err(invalid(format!("category {id} does not exist"))),
        }
    }

    /// Files a meal or plan under the category an edit names, or takes it out of the one
    /// it is in, and says so. Nothing happens when the edit names none or the same one.
    fn recategorise(
        &self,
        category_id: &mut Option<Uuid>,
        wanted: Option<Option<Uuid>>,
        scope: CategoryScope,
        lines: &mut Vec<String>,
    ) -> Result<()> {
        let Some(wanted) = wanted.filter(|wanted| wanted != category_id) else {
            return Ok(());
        };
        let line = match wanted {
            Some(id) => format!("Filed under {}", self.category_name(id, scope)?),
            None => match category_id.and_then(|id| self.category_name(id, scope).ok()) {
                Some(name) => format!("Taken out of {name}"),
                None => "Taken out of its category".to_owned(),
            },
        };
        lines.push(line);
        *category_id = wanted;
        Ok(())
    }
}

// the rules

impl<S, C, A> Mealprep<S, C, A>
where
    S: AssistantStore
        + CalendarStore
        + CategoryStore
        + DayPlanStore
        + MealStore
        + ProductStore
        + TargetsStore,
    C: Catalogue,
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

        // A question is answered in the prose and proposes nothing, which resolves to an
        // empty preview rather than a failure.
        let mut world = world;
        self.fetch_catalogue(&mut world, &proposal.operations)
            .await?;
        let resolved = resolve(&world, &proposal.request, &proposal.operations)?;
        Ok(preview(&world, proposal, &resolved))
    }
}

impl<S, C, A> Mealprep<S, C, A>
where
    S: AssistantStore
        + CalendarStore
        + CategoryStore
        + DayPlanStore
        + MealStore
        + ProductStore
        + TargetsStore,
    C: Catalogue,
{
    /// Applies the operations the user confirmed and records what it did.
    ///
    /// Everything is checked again before anything is written, so a proposal that no
    /// longer fits the data — a product deleted since, a date now in the past — fails
    /// without having touched a row.
    pub async fn apply_assistant_edit(&self, proposal: Proposal) -> Result<AssistantRecord> {
        let request = validate::assistant_request(proposal.request)?;
        let mut world = self.load_world().await?;
        // Fetched again rather than taken from the preview: the confirmation comes back
        // over the bridge, and what is written has to be what the catalogue says now.
        self.fetch_catalogue(&mut world, &proposal.operations)
            .await?;
        let resolved = resolve(&world, &request, &proposal.operations)?;
        if resolved.entities.is_empty() && resolved.days.is_empty() {
            return Err(invalid("there is nothing to apply".to_owned()));
        }

        // Entities first, so a date can be pointed at a plan or meal this proposal creates;
        // each create's provisional id is mapped to the one the store assigned.
        let mut ids = HashMap::new();
        let mut entity_changes = Vec::with_capacity(resolved.entities.len());
        for change in &resolved.entities {
            entity_changes.push(self.write_entity(change, &mut ids).await?);
        }

        let mut applied = Vec::with_capacity(resolved.days.len());
        for change in &resolved.days {
            applied.push(self.write_day(&remap_day(change, &ids)).await?);
        }

        let draft = AssistantRecordDraft {
            request,
            rationale: proposal.rationale,
            operations: proposal.operations,
            changes: applied,
            entity_changes,
        };
        self.store.record_assistant_request(&draft).await
    }

    /// Writes one product, meal or day plan, naming what earlier creates of the same
    /// proposal made by the ids the store gave them.
    async fn write_entity(
        &self,
        change: &EntityChange,
        ids: &mut HashMap<Uuid, Uuid>,
    ) -> Result<AppliedEntityChange> {
        let create = change.action == EntityAction::Create;
        let id = match &change.draft {
            EntityDraft::Product(draft) if create => {
                self.store.create_product(draft, None).await?.id
            }
            EntityDraft::Product(draft) => {
                let id = self.store.update_product(change.id, draft).await?.id;
                if let Some(code) = &change.link {
                    self.store.set_product_source(id, code).await?;
                }
                id
            }
            EntityDraft::Meal(draft) => {
                let draft = MealDraft {
                    servings: draft
                        .servings
                        .iter()
                        .map(|serving| Serving {
                            product_id: remap(serving.product_id, ids),
                            amount: serving.amount,
                        })
                        .collect(),
                    ..draft.clone()
                };
                if create {
                    self.store.create_meal(&draft).await?.id
                } else {
                    self.store.update_meal(change.id, &draft).await?.id
                }
            }
            EntityDraft::DayPlan(draft) => {
                let draft = DayPlanDraft {
                    items: remap_items(&draft.items, ids),
                    ..draft.clone()
                };
                if create {
                    self.store.create_day_plan(&draft).await?.id
                } else {
                    self.store.update_day_plan(change.id, &draft).await?.id
                }
            }
            EntityDraft::Targets(macros) => {
                self.store.set_targets(*macros).await?;
                Uuid::nil()
            }
        };
        if create {
            ids.insert(change.id, id);
        }

        Ok(AppliedEntityChange {
            kind: change.draft.kind(),
            action: change.action,
            id,
            label: change.draft.label().to_owned(),
            changes: change.changes.clone(),
        })
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

        let assignable = kept.iter().map(|plan| plan.id).collect();
        let plans: HashMap<Uuid, DayPlan> = kept
            .into_iter()
            .chain(copies)
            .map(|plan| (plan.id, plan))
            .collect();

        let meals = self.store.list_meals().await?;
        let meal_order = meals.iter().map(|meal| meal.id).collect();
        let products = self.store.list_products().await?;
        let product_order = products.iter().map(|product| product.id).collect();

        let mut categories = self.store.list_categories(CategoryScope::Meal).await?;
        categories.extend(self.store.list_categories(CategoryScope::DayPlan).await?);

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
            meal_order,
            product_order,
            meals,
            products,
            categories,
            catalogue: HashMap::new(),
        })
    }

    /// Fetches the catalogue entry each sync of the proposal names, so checking it needs
    /// nothing from outside. A sync with no code to fetch is left for checking to refuse.
    async fn fetch_catalogue(&self, world: &mut World, operations: &[Operation]) -> Result<()> {
        for operation in operations {
            let Operation::SyncProduct { product_id, code } = operation else {
                continue;
            };
            let code = code.clone().or_else(|| {
                world
                    .products
                    .get(product_id)
                    .and_then(|product| product.source_code.clone())
            });
            let Some(Ok(code)) = code.map(|code| validate::catalogue_code(&code)) else {
                continue;
            };
            if let Entry::Vacant(slot) = world.catalogue.entry(code) {
                let entry = self.catalogue.get_catalogue_entry(slot.key()).await?;
                slot.insert(entry);
            }
        }
        Ok(())
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

/// Checks every operation against the data and says what each entity and date it names
/// becomes.
///
/// Entity edits go first, products before meals before day plans, so each can name what
/// an earlier one creates; they change a copy of the data, which the day edits are then
/// checked against. Day edits are checked in the order they were proposed and compose:
/// two corrections to the same day both land, rather than the second overwriting the
/// first's result.
fn resolve(world: &World, request: &str, operations: &[Operation]) -> Result<Resolved> {
    let mut after = world.clone();
    let mut created = HashSet::new();
    let mut entity_operations: Vec<&Operation> = operations
        .iter()
        .filter(|operation| operation.entity().is_some())
        .collect();
    // Stable, so two edits to one kind keep the order they were proposed in.
    entity_operations.sort_by_key(|operation| operation.entity());
    let mut entities = Vec::with_capacity(entity_operations.len());
    for operation in entity_operations {
        entities.push(edit_entity(&mut after, operation, &mut created)?);
    }

    let past_allowed = validate::past_dates_requested(request, after.today);
    let mut changes: BTreeMap<Date, DayChange> = BTreeMap::new();
    for operation in operations {
        for date in checked_dates(&after, operation, past_allowed)? {
            let change = changes
                .entry(date)
                .or_insert_with(|| initial_change(&after, date));
            edit(&after, operation, change)?;
        }
    }

    // A date an operation named but did not change is not a change to confirm.
    let days = changes
        .into_values()
        .filter(|change| !change.changes.is_empty())
        .collect();
    Ok(Resolved {
        entities,
        days,
        after,
    })
}

/// What one entity edit creates or changes, applied to `world` so the edits after it see it.
///
/// This is the only place an entity edit's meaning is written down, as [`edit`] is for day
/// edits. What it produces is the whole entity to write, checked by the same rules the
/// screens' own forms are.
fn edit_entity(
    world: &mut World,
    operation: &Operation,
    created: &mut HashSet<Uuid>,
) -> Result<EntityChange> {
    let (action, id, draft, changes, link) = match operation {
        Operation::CreateProduct { new_id, product } => {
            claim_new_id(world, created, *new_id)?;
            let draft = validate::product(product.clone())?;
            put_product(world, *new_id, &draft, None, None);
            world.product_order.push(*new_id);
            let line = format!("New product, figures per 100 {}", draft.unit);
            (
                EntityAction::Create,
                *new_id,
                EntityDraft::Product(draft),
                vec![line],
                None,
            )
        }

        Operation::UpdateProduct {
            product_id,
            changes,
        } => {
            refuse_created(created, *product_id, "product")?;
            let current = world.product(*product_id)?.clone();
            let mut lines = Vec::new();
            let mut draft = product_draft(&current);
            if let Some(name) = changes.name.as_ref().filter(|name| **name != draft.name) {
                lines.push(format!("Renamed from {} to {name}", draft.name));
                draft.name = name.clone();
            }
            set_brand(&mut draft.brand, changes.brand.clone(), &mut lines);
            if let Some(unit) = changes.unit.filter(|unit| *unit != draft.unit) {
                lines.push(format!(
                    "Figures now per 100 {unit} instead of per 100 {}",
                    draft.unit
                ));
                draft.unit = unit;
            }
            let per = format!("per 100 {}", draft.unit);
            for (field, wanted) in [
                (MacroField::EnergyKcal, changes.energy_kcal),
                (MacroField::ProteinG, changes.protein_g),
                (MacroField::FatG, changes.fat_g),
                (MacroField::CarbohydratesG, changes.carbohydrates_g),
            ] {
                set_macro(&mut draft.macros, field, wanted, &per, &mut lines);
            }
            match changes.cost {
                Some(Some(cost)) if draft.cost != Some(cost) => {
                    lines.push(match draft.cost {
                        Some(was) => format!("Cost {per} {was} → {cost}"),
                        None => format!("Cost {per} {cost}"),
                    });
                    draft.cost = Some(cost);
                }
                Some(None) if draft.cost.is_some() => {
                    lines.push("Cost removed".to_owned());
                    draft.cost = None;
                }
                _ => {}
            }
            for (key, wanted) in &changes.nutrients {
                set_nutrient(&mut draft.nutrients, key, *wanted, &per, &mut lines);
            }
            if lines.is_empty() {
                return Err(invalid(format!(
                    "the update changes nothing about {}",
                    current.name
                )));
            }
            let draft = validate::product(draft)?;
            put_product(world, *product_id, &draft, Some(&current), None);
            (
                EntityAction::Update,
                *product_id,
                EntityDraft::Product(draft),
                lines,
                None,
            )
        }

        Operation::SyncProduct { product_id, code } => {
            refuse_created(created, *product_id, "product")?;
            let current = world.product(*product_id)?.clone();
            let Some(code) = code.as_deref().or(current.source_code.as_deref()) else {
                return Err(invalid(format!(
                    "{} was entered by hand, so there is no Open Food Facts product to sync it \
                     from; give its barcode",
                    current.name
                )));
            };
            let code = validate::catalogue_code(code)?;
            let entry = world
                .catalogue
                .get(&code)
                .cloned()
                .ok_or_else(|| Error::Internal {
                    reason: format!("catalogue entry {code} was not fetched before checking"),
                })?;
            let (draft, mut lines) = synced(&current, &entry)?;
            let link = (current.source_code.as_deref() != Some(code.as_str())).then_some(code);
            if let Some(code) = &link {
                lines.insert(0, format!("Linked to Open Food Facts product {code}"));
            }
            put_product(world, *product_id, &draft, Some(&current), link.clone());
            (
                EntityAction::Update,
                *product_id,
                EntityDraft::Product(draft),
                lines,
                link,
            )
        }

        Operation::UpdateTargets { changes } => {
            let wanted = [
                (MacroField::EnergyKcal, changes.energy_kcal),
                (MacroField::ProteinG, changes.protein_g),
                (MacroField::FatG, changes.fat_g),
                (MacroField::CarbohydratesG, changes.carbohydrates_g),
            ];
            // With none set yet there is nothing to keep, so all four have to be given.
            let complete = wanted.iter().all(|(_, value)| value.is_some());
            let Some(mut macros) = world.targets.or(complete.then_some(Macros::ZERO)) else {
                return Err(invalid(
                    "there are no targets yet, so all four figures are needed".to_owned(),
                ));
            };
            let mut lines = Vec::new();
            for (field, value) in wanted {
                set_macro(&mut macros, field, value, "a day", &mut lines);
            }
            if lines.is_empty() {
                return Err(invalid("the update changes none of the targets".to_owned()));
            }
            let macros = validate::targets(macros)?;
            world.targets = Some(macros);
            (
                EntityAction::Update,
                Uuid::nil(),
                EntityDraft::Targets(macros),
                lines,
                None,
            )
        }

        Operation::CreateMeal { new_id, meal } => {
            claim_new_id(world, created, *new_id)?;
            let draft = validate::meal(meal.clone())?;
            for serving in &draft.servings {
                world.product(serving.product_id)?;
            }
            let mut lines = vec![format!("Made of {}", world.servings_text(&draft.servings))];
            if let Some(id) = draft.category_id {
                let name = world.category_name(id, CategoryScope::Meal)?;
                lines.push(format!("Filed under {name}"));
            }
            put_meal(world, *new_id, &draft, None);
            world.meal_order.push(*new_id);
            (
                EntityAction::Create,
                *new_id,
                EntityDraft::Meal(draft),
                lines,
                None,
            )
        }

        Operation::UpdateMeal { meal_id, changes } => {
            refuse_created(created, *meal_id, "meal")?;
            let current = world
                .meals
                .get(meal_id)
                .map(|meal| meal.entity.clone())
                .ok_or_else(|| invalid(format!("meal {meal_id} does not exist")))?;
            let mut lines = Vec::new();
            let mut draft = MealDraft {
                label: current.label.clone(),
                category_id: current.category_id,
                servings: current.servings.clone(),
            };
            if let Some(label) = changes
                .label
                .as_ref()
                .filter(|label| **label != draft.label)
            {
                lines.push(format!("Renamed from {} to {label}", draft.label));
                draft.label = label.clone();
            }
            world.recategorise(
                &mut draft.category_id,
                changes.category_id,
                CategoryScope::Meal,
                &mut lines,
            )?;
            if let Some(servings) = changes
                .servings
                .as_ref()
                .filter(|servings| **servings != draft.servings)
            {
                for serving in servings {
                    world.product(serving.product_id)?;
                }
                lines.push(format!(
                    "Now {}, instead of {}",
                    world.servings_text(servings),
                    world.servings_text(&draft.servings)
                ));
                draft.servings = servings.clone();
            }
            if lines.is_empty() {
                return Err(invalid(format!(
                    "the update changes nothing about {}",
                    current.label
                )));
            }
            let draft = validate::meal(draft)?;
            put_meal(world, *meal_id, &draft, Some(&current));
            (
                EntityAction::Update,
                *meal_id,
                EntityDraft::Meal(draft),
                lines,
                None,
            )
        }

        Operation::CreateDayPlan { new_id, day_plan } => {
            claim_new_id(world, created, *new_id)?;
            let draft = validate::day_plan(day_plan.clone())?;
            world.check_items(&draft.items)?;
            let mut lines = vec![format!("Holds {}", world.items_text(&draft.items))];
            if let Some(id) = draft.category_id {
                let name = world.category_name(id, CategoryScope::DayPlan)?;
                lines.push(format!("Filed under {name}"));
            }
            world.plans.insert(
                *new_id,
                DayPlan {
                    id: *new_id,
                    label: draft.label.clone(),
                    category_id: draft.category_id,
                    source_plan_id: None,
                    items: draft.items.clone(),
                    created_at: OffsetDateTime::UNIX_EPOCH,
                    updated_at: OffsetDateTime::UNIX_EPOCH,
                },
            );
            world.assignable.push(*new_id);
            (
                EntityAction::Create,
                *new_id,
                EntityDraft::DayPlan(draft),
                lines,
                None,
            )
        }

        Operation::UpdateDayPlan {
            day_plan_id,
            changes,
        } => {
            refuse_created(created, *day_plan_id, "day plan")?;
            let current = world.assignable_plan(*day_plan_id)?.clone();
            let mut lines = Vec::new();
            let mut draft = DayPlanDraft {
                label: current.label.clone(),
                category_id: current.category_id,
                items: current.items.clone(),
            };
            if let Some(label) = changes
                .label
                .as_ref()
                .filter(|label| **label != draft.label)
            {
                lines.push(format!("Renamed from {} to {label}", draft.label));
                draft.label = label.clone();
            }
            world.recategorise(
                &mut draft.category_id,
                changes.category_id,
                CategoryScope::DayPlan,
                &mut lines,
            )?;
            if let Some(items) = changes
                .items
                .as_ref()
                .filter(|items| **items != draft.items)
            {
                world.check_items(items)?;
                lines.push(format!(
                    "Now {}, instead of {}",
                    world.items_text(items),
                    world.items_text(&draft.items)
                ));
                draft.items = items.clone();
            }
            if lines.is_empty() {
                return Err(invalid(format!(
                    "the update changes nothing about {}",
                    current.label
                )));
            }
            let draft = validate::day_plan(draft)?;
            world.plans.insert(
                *day_plan_id,
                DayPlan {
                    label: draft.label.clone(),
                    category_id: draft.category_id,
                    items: draft.items.clone(),
                    ..current
                },
            );
            (
                EntityAction::Update,
                *day_plan_id,
                EntityDraft::DayPlan(draft),
                lines,
                None,
            )
        }

        Operation::AdjustProductAmount { .. }
        | Operation::AddItem { .. }
        | Operation::RemoveItem { .. }
        | Operation::SwapMeal { .. }
        | Operation::AssignPlan { .. }
        | Operation::ClearPlan { .. } => {
            return Err(Error::Internal {
                reason: format!("{} is not an entity edit", operation.kind().name()),
            });
        }
    };

    Ok(EntityChange {
        action,
        id,
        draft,
        changes,
        link,
    })
}

/// Sets one of the four figures and says so, when the update gives a different one.
fn set_macro(
    macros: &mut Macros,
    field: MacroField,
    wanted: Option<f64>,
    per: &str,
    lines: &mut Vec<String>,
) {
    let (name, unit, figure) = match field {
        MacroField::EnergyKcal => ("Energy", "kcal", &mut macros.energy_kcal),
        MacroField::ProteinG => ("Protein", "g", &mut macros.protein_g),
        MacroField::FatG => ("Fat", "g", &mut macros.fat_g),
        MacroField::CarbohydratesG => ("Carbs", "g", &mut macros.carbohydrates_g),
    };
    if let Some(wanted) = wanted.filter(|wanted| wanted != figure) {
        lines.push(format!("{name} {per} {figure} {unit} → {wanted} {unit}"));
        *figure = wanted;
    }
}

/// Sets, replaces or removes a product's brand and says so, when that changes it.
fn set_brand(brand: &mut Option<String>, wanted: Option<Option<String>>, lines: &mut Vec<String>) {
    match wanted {
        Some(Some(wanted)) if brand.as_ref() != Some(&wanted) => {
            lines.push(format!("Brand {wanted}"));
            *brand = Some(wanted);
        }
        Some(None) if brand.is_some() => {
            lines.push("Brand removed".to_owned());
            *brand = None;
        }
        _ => {}
    }
}

/// Sets or removes one nutrient beyond the macros and says so, when that changes it.
fn set_nutrient(
    nutrients: &mut Nutrients,
    key: &str,
    wanted: Option<f64>,
    per: &str,
    lines: &mut Vec<String>,
) {
    let key = key.trim();
    let name = nutrient_name(key);
    let line = match (nutrients.get(key).copied(), wanted) {
        (Some(was), Some(value)) if was != value => {
            format!("{name} {per} {} → {}", mass(was), mass(value))
        }
        (None, Some(value)) => format!("{name} {per} {}", mass(value)),
        (Some(_), None) => format!("{name} removed"),
        _ => return,
    };
    lines.push(line);
    match wanted {
        Some(value) => nutrients.insert(key.to_owned(), value),
        None => nutrients.remove(key),
    };
}

/// What a product becomes when its figures are replaced by the catalogue entry's, and
/// what that changes.
///
/// The name and the price are the user's and stay. A macro the catalogue lacks keeps the
/// figure the product has rather than becoming zero, and the catalogue's own doubts about
/// its figures are passed on, because the user is about to take them on trust.
fn synced(current: &Product, entry: &CatalogueEntry) -> Result<(ProductDraft, Vec<String>)> {
    let mut draft = product_draft(current);
    let mut lines = Vec::new();
    if let Some(brand) = &entry.brand {
        set_brand(&mut draft.brand, Some(Some(brand.clone())), &mut lines);
    }
    if entry.unit != draft.unit {
        lines.push(format!(
            "Figures now per 100 {} instead of per 100 {}",
            entry.unit, draft.unit
        ));
        draft.unit = entry.unit;
    }
    let per = format!("per 100 {}", draft.unit);
    for (field, value) in [
        (MacroField::EnergyKcal, entry.macros.energy_kcal),
        (MacroField::ProteinG, entry.macros.protein_g),
        (MacroField::FatG, entry.macros.fat_g),
        (MacroField::CarbohydratesG, entry.macros.carbohydrates_g),
    ] {
        let wanted = (!entry.missing_macros.contains(&field)).then_some(value);
        set_macro(&mut draft.macros, field, wanted, &per, &mut lines);
    }
    let keys: Vec<String> = current
        .nutrients
        .keys()
        .chain(entry.nutrients.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    for key in keys {
        let wanted = entry.nutrients.get(&key).copied();
        set_nutrient(&mut draft.nutrients, &key, wanted, &per, &mut lines);
    }
    if lines.is_empty() {
        return Err(invalid(format!(
            "{} already matches Open Food Facts",
            current.name
        )));
    }
    if !entry.missing_macros.is_empty() {
        lines.push("Open Food Facts lacks some of the four figures; those are kept".to_owned());
    }
    lines.extend(
        entry
            .warnings
            .iter()
            .map(|warning| format!("Open Food Facts notes: {warning}")),
    );
    Ok((validate::product(draft)?, lines))
}

/// The product as a draft, every field as it is, for an edit to start from.
fn product_draft(product: &Product) -> ProductDraft {
    ProductDraft {
        name: product.name.clone(),
        unit: product.unit,
        macros: product.macros,
        nutrients: product.nutrients.clone(),
        cost: product.cost,
        brand: product.brand.clone(),
    }
}

/// Puts a product into the data and re-derives the meals that may hold it.
fn put_product(
    world: &mut World,
    id: Uuid,
    draft: &ProductDraft,
    current: Option<&Product>,
    link: Option<String>,
) {
    let product = Product {
        id,
        name: draft.name.clone(),
        unit: draft.unit,
        macros: draft.macros,
        nutrients: draft.nutrients.clone(),
        cost: draft.cost,
        brand: draft.brand.clone(),
        source_code: link.or_else(|| current.and_then(|product| product.source_code.clone())),
        created_at: current.map_or(OffsetDateTime::UNIX_EPOCH, |product| product.created_at),
        updated_at: current.map_or(OffsetDateTime::UNIX_EPOCH, |product| product.updated_at),
    };
    world.products.insert(id, product);
    world.rederive_meals();
}

/// "Vitamin c" for `vitamin-c`: the catalogue's key, readable in a change line.
fn nutrient_name(key: &str) -> String {
    let words = key.replace(['-', '_'], " ");
    let mut chars = words.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// A nutrient figure, stored in grams, in the unit a label would print it in.
fn mass(grams: f64) -> String {
    let (value, unit) = if grams == 0.0 || grams.abs() >= 1.0 {
        (grams, "g")
    } else if grams.abs() >= 0.001 {
        (grams * 1e3, "mg")
    } else {
        (grams * 1e6, "µg")
    };
    format!("{} {unit}", significant(value))
}

/// Four significant figures. A nutrient in grams runs from tens of grams of fibre to
/// micrograms of B12, so a fixed number of decimals would either pad the one or zero the
/// other.
fn significant(value: f64) -> f64 {
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    // A float's decimal exponent is within ±330, so the cast cannot truncate.
    #[allow(clippy::cast_possible_truncation)]
    let magnitude = value.abs().log10().floor() as i32;
    let scale = 10f64.powi(3 - magnitude);
    (value * scale).round() / scale
}

/// Puts a meal into the data, with its totals derived as a stored one's would be.
fn put_meal(world: &mut World, id: Uuid, draft: &MealDraft, current: Option<&Meal>) {
    let meal = Meal {
        id,
        label: draft.label.clone(),
        category_id: draft.category_id,
        servings: draft.servings.clone(),
        created_at: current.map_or(OffsetDateTime::UNIX_EPOCH, |meal| meal.created_at),
        updated_at: current.map_or(OffsetDateTime::UNIX_EPOCH, |meal| meal.updated_at),
    };
    let derived = derive::derive_meal(meal, &world.products);
    world.meals.insert(id, derived);
}

/// Reserves the id a create names, which must not belong to anything already there.
fn claim_new_id(world: &World, created: &mut HashSet<Uuid>, id: Uuid) -> Result<()> {
    let taken = world.products.contains_key(&id)
        || world.meals.contains_key(&id)
        || world.plans.contains_key(&id)
        || !created.insert(id);
    if taken {
        return Err(invalid(format!(
            "the new id {id} is already in use; each new item needs an id of its own"
        )));
    }
    Ok(())
}

/// An update names something already there; what this proposal creates is set in full by
/// its create, so updating it as well would only split one edit in two.
fn refuse_created(created: &HashSet<Uuid>, id: Uuid, what: &str) -> Result<()> {
    if created.contains(&id) {
        return Err(invalid(format!(
            "the new {what} {id} is set in full where it is created, not updated after"
        )));
    }
    Ok(())
}

fn remap(id: Uuid, ids: &HashMap<Uuid, Uuid>) -> Uuid {
    ids.get(&id).copied().unwrap_or(id)
}

fn remap_items(items: &[DayPlanItem], ids: &HashMap<Uuid, Uuid>) -> Vec<DayPlanItem> {
    items
        .iter()
        .map(|item| match item {
            DayPlanItem::Meal { meal_id } => DayPlanItem::Meal {
                meal_id: remap(*meal_id, ids),
            },
            DayPlanItem::Product { product_id, amount } => DayPlanItem::Product {
                product_id: remap(*product_id, ids),
                amount: *amount,
            },
        })
        .collect()
}

/// The day change with every provisional id replaced by the one the store assigned.
fn remap_day(change: &DayChange, ids: &HashMap<Uuid, Uuid>) -> DayChange {
    let mut change = change.clone();
    if let Some(plan) = change.plan.as_mut() {
        plan.current_plan_id = remap(plan.current_plan_id, ids);
        plan.source_plan_id = remap(plan.source_plan_id, ids);
        plan.items = remap_items(&plan.items, ids);
    }
    change
}

/// The dates an operation may act on, or why it may not act on them.
///
/// Three bounds, each its own refusal rather than a silent trim: the range has to run
/// forwards and stay short, it has to lie inside the window the assistant can see, and a
/// date before today needs the request to have asked for it in so many words.
fn checked_dates(world: &World, operation: &Operation, past_allowed: bool) -> Result<Vec<Date>> {
    // An entity edit names no date; it was resolved before the days.
    let Some(range) = operation.dates() else {
        return Ok(Vec::new());
    };
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

        Operation::CreateProduct { .. }
        | Operation::UpdateProduct { .. }
        | Operation::CreateMeal { .. }
        | Operation::UpdateMeal { .. }
        | Operation::CreateDayPlan { .. }
        | Operation::UpdateDayPlan { .. }
        | Operation::SyncProduct { .. }
        | Operation::UpdateTargets { .. } => {
            return Err(Error::Internal {
                reason: format!("{} names no date", operation.kind().name()),
            });
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

/// What the user reads before confirming: the things created or changed, the dates that
/// change and what changes on each, and where that leaves the day against the targets.
fn preview(world: &World, proposal: Proposal, resolved: &Resolved) -> Preview {
    let after_world = &resolved.after;
    let entities = resolved
        .entities
        .iter()
        .map(|change| {
            let kind = change.draft.kind();
            let macros = match &change.draft {
                EntityDraft::Product(draft) => draft.macros,
                EntityDraft::Meal(_) => after_world
                    .meals
                    .get(&change.id)
                    .map_or(Macros::ZERO, |meal| meal.macros),
                EntityDraft::DayPlan(draft) => after_world.macros_of(&draft.items),
                EntityDraft::Targets(macros) => *macros,
            };
            PreviewEntity {
                kind,
                action: change.action,
                label: change.draft.label().to_owned(),
                changes: change.changes.clone(),
                macros,
                dates_using: world.dates_using(kind, change.id),
            }
        })
        .collect();

    let days = resolved
        .days
        .iter()
        .map(|change| {
            let before = world
                .plan_on(change.date)
                .map_or(Macros::ZERO, |plan| world.macros_of(&plan.items));
            let after = change
                .plan
                .as_ref()
                .map_or(Macros::ZERO, |plan| after_world.macros_of(&plan.items));
            PreviewDay {
                date: change.date,
                label: change.plan.as_ref().map(|plan| plan.label.clone()),
                changes: change.changes.clone(),
                before,
                after,
                // Against the target rather than against `before`: the target is the
                // figure the day is planned towards, and the deviation is what to read.
                // The targets as they would be, when the same proposal changes them.
                target_delta: after_world.targets.map(|target| after + target.scale(-1.0)),
                detaches: change.detaches(),
            }
        })
        .collect();

    Preview {
        proposal,
        entities,
        days,
        targets: after_world.targets,
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
    use crate::domain::{DateRange, DayPlanPatch, MealPatch, Nutrients, ProductPatch, Unit};

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
            assignable: vec![training.id, rest.id],
            meal_order: vec![porridge.id, omelette.id],
            product_order: vec![whey.id, banana.id],
            plans: [training.clone(), rest.clone()]
                .into_iter()
                .map(|plan| (plan.id, plan))
                .collect(),
            meals,
            products,
            categories: Vec::new(),
            catalogue: HashMap::new(),
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

    fn resolve_days(
        world: &World,
        request: &str,
        operations: &[Operation],
    ) -> Result<Vec<DayChange>> {
        resolve(world, request, operations).map(|resolved| resolved.days)
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

        let changes =
            resolve_days(&f.world, "next 4 days I have 40 g less whey", &operations).unwrap();
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
        let changes = resolve_days(&f.world, "today 40 g less whey", &operations).unwrap();

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

        let Err(Error::Invalid { reason, .. }) =
            resolve_days(&f.world, "40 g less whey", &operations)
        else {
            panic!("a past date should be refused");
        };
        assert!(reason.contains("has passed"), "{reason}");

        // The same operation, from a request that says so, goes through.
        let changes =
            resolve_days(&f.world, "yesterday I had 40 g less whey", &operations).unwrap();
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
                    resolve_days(&f.world, "clear those days", &operations),
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
            resolve_days(&f.world, "less of that", &unknown),
            Err(Error::Invalid { .. })
        ));

        // 60 g of whey cannot lose 60: an item is removed, never left at nothing.
        let impossible = [Operation::AdjustProductAmount {
            dates: DateRange::just(TODAY),
            product_id: f.whey,
            delta_amount: -60.0,
        }];
        let Err(Error::Invalid { reason, .. }) =
            resolve_days(&f.world, "no whey today", &impossible)
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
        let Err(Error::Invalid { reason, .. }) = resolve_days(&f.world, "less banana", &nested)
        else {
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
                item: ItemRef::Product { product_id: f.whey },
            },
        ];

        for operation in ambiguous {
            let Err(Error::Invalid { reason, .. }) =
                resolve_days(&f.world, "40 g less whey", &[operation])
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

        let changes = resolve_days(
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

        let changes =
            resolve_days(&f.world, "rest day tomorrow with a shake", &operations).unwrap();
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
        let changes = resolve_days(&f.world, "nothing planned today", &operations).unwrap();

        assert!(changes[0].plan.is_none());
        assert!(!changes[0].detaches());
        assert_eq!(changes[0].previous_day_plan_id, Some(f.training));
        assert_eq!(changes[0].previous_label.as_deref(), Some("Training day"));

        // Clearing a day that has nothing is a refusal, not a no-op.
        let empty = [Operation::ClearPlan {
            dates: DateRange::just(TODAY + Duration::days(30)),
        }];
        assert!(matches!(
            resolve_days(&f.world, "nothing planned then", &empty),
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
        let resolved = resolve(&f.world, "40 g less whey today", &operations).unwrap();
        let proposal = Proposal {
            request: "40 g less whey today".to_owned(),
            rationale: "Dropped the shake".to_owned(),
            operations: operations.to_vec(),
        };

        let preview = preview(&f.world, proposal, &resolved);
        assert!(preview.entities.is_empty());
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

    fn draft_product(name: &str) -> ProductDraft {
        ProductDraft {
            name: name.to_owned(),
            unit: Unit::Gram,
            macros: Macros {
                energy_kcal: 370.0,
                fat_g: 7.0,
                protein_g: 13.0,
                carbohydrates_g: 60.0,
            },
            nutrients: Nutrients::new(),
            cost: None,
            brand: None,
        }
    }

    #[test]
    fn creates_a_product_a_meal_and_a_plan_that_name_each_other_and_puts_it_on_a_date() {
        let f = fixture();
        let oats = Uuid::new_v4();
        let bowl = Uuid::new_v4();
        let plan = Uuid::new_v4();
        // Proposed out of order: the plan first, then the meal, then the product it needs.
        let operations = [
            Operation::AssignPlan {
                dates: DateRange::just(TODAY),
                day_plan_id: plan,
            },
            Operation::CreateDayPlan {
                new_id: plan,
                day_plan: DayPlanDraft {
                    label: "Oat day".to_owned(),
                    category_id: None,
                    items: vec![DayPlanItem::Meal { meal_id: bowl }],
                },
            },
            Operation::CreateMeal {
                new_id: bowl,
                meal: MealDraft {
                    label: "Oat bowl".to_owned(),
                    category_id: None,
                    servings: vec![
                        Serving {
                            product_id: oats,
                            amount: 80.0,
                        },
                        Serving {
                            product_id: f.whey,
                            amount: 30.0,
                        },
                    ],
                },
            },
            Operation::CreateProduct {
                new_id: oats,
                product: draft_product("Oats"),
            },
        ];

        let resolved = resolve(&f.world, "an oat day today", &operations).unwrap();
        let kinds: Vec<_> = resolved
            .entities
            .iter()
            .map(|change| (change.draft.kind(), change.action))
            .collect();
        assert_eq!(
            kinds,
            vec![
                (EntityKind::Product, EntityAction::Create),
                (EntityKind::Meal, EntityAction::Create),
                (EntityKind::DayPlan, EntityAction::Create),
            ]
        );
        assert_eq!(
            resolved.entities[1].changes,
            vec!["Made of 80 g Oats, 30 g Whey"]
        );

        assert_eq!(resolved.days.len(), 1);
        let day = resolved.days[0].plan.as_ref().unwrap();
        assert_eq!(day.current_plan_id, plan);
        assert_eq!(day.label, "Oat day");

        // The new things are counted as they would be once written.
        let preview = preview(
            &f.world,
            Proposal {
                request: "an oat day today".to_owned(),
                rationale: String::new(),
                operations: operations.to_vec(),
            },
            &resolved,
        );
        let expected = 0.8 * 370.0 + 0.3 * 380.0;
        assert!((preview.entities[1].macros.energy_kcal - expected).abs() < 1e-9);
        assert!((preview.days[0].after.energy_kcal - expected).abs() < 1e-9);

        // Once applied, every provisional id is the one the store gave it.
        let ids: HashMap<Uuid, Uuid> = [
            (oats, Uuid::new_v4()),
            (bowl, Uuid::new_v4()),
            (plan, Uuid::new_v4()),
        ]
        .into_iter()
        .collect();
        let written = remap_day(&resolved.days[0], &ids);
        assert_eq!(written.plan.unwrap().current_plan_id, ids[&plan]);
    }

    #[test]
    fn updating_a_product_says_what_changed_and_carries_into_the_days_that_hold_it() {
        let f = fixture();
        let operations = [
            Operation::UpdateProduct {
                product_id: f.whey,
                changes: ProductPatch {
                    energy_kcal: Some(400.0),
                    cost: Some(Some(2.5)),
                    ..ProductPatch::default()
                },
            },
            Operation::AdjustProductAmount {
                dates: DateRange::just(TODAY),
                product_id: f.whey,
                delta_amount: -10.0,
            },
        ];

        let resolved = resolve(
            &f.world,
            "whey is 400 kcal now, and 10 g less today",
            &operations,
        )
        .unwrap();
        let change = &resolved.entities[0];
        assert_eq!(
            change.changes,
            vec![
                "Energy per 100 g 380 kcal → 400 kcal".to_owned(),
                "Cost per 100 g 1 → 2.5".to_owned(),
            ]
        );
        let EntityDraft::Product(draft) = &change.draft else {
            panic!("a product update writes a product");
        };
        // What the patch left out is kept, not cleared.
        assert_eq!(draft.name, "Whey");
        assert!((draft.macros.protein_g - 20.0).abs() < 1e-9);

        let preview = preview(
            &f.world,
            Proposal {
                request: String::new(),
                rationale: String::new(),
                operations: operations.to_vec(),
            },
            &resolved,
        );
        // Every date in the fixture holds whey loose on the training plan.
        assert_eq!(preview.entities[0].dates_using, 8);
        let day = &preview.days[0];
        assert!((day.after.energy_kcal - (89.0 + 0.5 * 400.0)).abs() < 1e-9);
        assert!((day.before.energy_kcal - (89.0 + 0.6 * 380.0)).abs() < 1e-9);
    }

    #[test]
    fn updating_a_plan_or_meal_reaches_every_date_on_it() {
        let f = fixture();
        let operations = [
            Operation::UpdateMeal {
                meal_id: f.porridge,
                changes: MealPatch {
                    label: Some("Banana porridge".to_owned()),
                    ..MealPatch::default()
                },
            },
            Operation::UpdateDayPlan {
                day_plan_id: f.training,
                changes: DayPlanPatch {
                    items: Some(vec![DayPlanItem::Meal {
                        meal_id: f.omelette,
                    }]),
                    ..DayPlanPatch::default()
                },
            },
        ];
        let resolved = resolve(&f.world, "omelette on training days", &operations).unwrap();
        assert!(resolved.days.is_empty(), "no single date was edited");
        assert_eq!(
            resolved.entities[0].changes,
            vec!["Renamed from Porridge to Banana porridge"]
        );
        assert_eq!(
            resolved.entities[1].changes,
            vec!["Now Omelette, instead of Banana porridge, 60 g Whey"]
        );
        assert_eq!(f.world.dates_using(EntityKind::DayPlan, f.training), 8);
        assert_eq!(f.world.dates_using(EntityKind::Meal, f.porridge), 8);
        assert_eq!(f.world.dates_using(EntityKind::Product, f.banana), 8);
        assert_eq!(f.world.dates_using(EntityKind::Meal, f.omelette), 0);
    }

    #[test]
    fn refuses_entity_edits_that_do_not_fit_the_data() {
        let f = fixture();
        let refused = |operations: &[Operation], needle: &str| {
            let Err(Error::Invalid { reason, .. }) = resolve(&f.world, "edit", operations) else {
                panic!("{operations:?} should be refused");
            };
            assert!(
                reason.contains(needle),
                "{reason:?} should mention {needle:?}"
            );
        };

        // A new id that is already somebody's.
        refused(
            &[Operation::CreateProduct {
                new_id: f.whey,
                product: draft_product("Oats"),
            }],
            "already in use",
        );
        // A meal made of a product that does not exist.
        refused(
            &[Operation::CreateMeal {
                new_id: Uuid::new_v4(),
                meal: MealDraft {
                    label: "Mystery".to_owned(),
                    category_id: None,
                    servings: vec![Serving {
                        product_id: Uuid::new_v4(),
                        amount: 50.0,
                    }],
                },
            }],
            "does not exist",
        );
        // An update that changes nothing.
        refused(
            &[Operation::UpdateProduct {
                product_id: f.whey,
                changes: ProductPatch {
                    name: Some("Whey".to_owned()),
                    ..ProductPatch::default()
                },
            }],
            "changes nothing",
        );
        // A date's own copy is not a plan the user keeps.
        let mut world = f.world.clone();
        let copy = day_plan("Training day", Vec::new(), Some(f.training));
        world.plans.insert(copy.id, copy.clone());
        let Err(Error::Invalid { reason, .. }) = resolve(
            &world,
            "edit",
            &[Operation::UpdateDayPlan {
                day_plan_id: copy.id,
                changes: DayPlanPatch {
                    label: Some("Other".to_owned()),
                    ..DayPlanPatch::default()
                },
            }],
        ) else {
            panic!("a copy cannot be updated");
        };
        assert!(reason.contains("own copy"));
        // A figure the product rules refuse, through the update as through the form.
        refused(
            &[Operation::UpdateProduct {
                product_id: f.whey,
                changes: ProductPatch {
                    protein_g: Some(-5.0),
                    ..ProductPatch::default()
                },
            }],
            "",
        );
    }

    #[test]
    fn a_key_is_never_reported_in_full() {
        assert_eq!(key_status(None), AssistantKeyStatus::default());
        let status = key_status(Some("sk-ant-api03-secret-tail"));
        assert!(status.present);
        assert_eq!(status.hint.as_deref(), Some("tail"));
    }
}
