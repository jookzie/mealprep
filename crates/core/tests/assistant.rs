//! Drives the assistant rules end to end with no database and no model.
//!
//! The store is a map in memory and the assistant is a canned answer, so what is under
//! test is only the rules: that the proposal is checked rather than trusted, that a dated
//! edit detaches the date it names and nothing else, and that applying records enough to
//! undo it by hand.
//!
//! The store fake implements the whole of every trait the rules ask for. The methods the
//! assistant never reaches answer `Internal` rather than a plausible value, so a rule that
//! starts using one fails loudly here instead of being quietly tested against a stub.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard},
};

use mealprep_core::{
    Entity, Error, Result,
    domain::{
        AssistantContext, AssistantRecord, AssistantRecordDraft, CalendarDay, Category,
        CategoryDraft, CategoryScope, DateRange, DayPlan, DayPlanDraft, DayPlanItem, ItemRef,
        Macros, Meal, MealDraft, Nutrients, Operation, Product, ProductDraft, Proposal,
        ResolvedDayPlan, ResolvedDayPlanItem, Targets, Unit,
    },
    service::Mealprep,
    store::{
        Assistant, AssistantStore, CalendarStore, CategoryStore, DayPlanStore, MealStore,
        ProductStore, TargetsStore,
    },
};
use time::{Date, Duration, OffsetDateTime};
use uuid::Uuid;

// the fakes

/// What the rules handed the assistant: the key, the request and the context.
type Seen = Arc<Mutex<Vec<(String, String, AssistantContext)>>>;

/// A canned answer, so the rules can be driven without a model or a network.
struct CannedAssistant {
    operations: Vec<Operation>,
    seen: Seen,
}

impl Assistant for CannedAssistant {
    async fn propose(
        &self,
        key: &str,
        request: &str,
        context: &AssistantContext,
    ) -> Result<Proposal> {
        self.seen
            .lock()
            .expect("the canned assistant is never poisoned")
            .push((key.to_owned(), request.to_owned(), context.clone()));
        Ok(Proposal {
            // Deliberately not the request the rules passed in: they keep their own.
            request: "something the user never typed".to_owned(),
            rationale: "Adjusted the days you named.".to_owned(),
            operations: self.operations.clone(),
        })
    }
}

#[derive(Default)]
struct State {
    products: Vec<Product>,
    meals: Vec<Meal>,
    plans: Vec<DayPlan>,
    calendar: BTreeMap<Date, Uuid>,
    targets: Option<Targets>,
    key: Option<String>,
    log: Vec<AssistantRecord>,
}

#[derive(Default)]
struct FakeStore(Mutex<State>);

impl FakeStore {
    fn state(&self) -> MutexGuard<'_, State> {
        self.0.lock().expect("the fake store is never poisoned")
    }
}

/// What a method the assistant rules do not use answers.
fn unused(method: &str) -> Error {
    Error::Internal {
        reason: format!("the assistant rules should not call {method}"),
    }
}

fn now() -> OffsetDateTime {
    OffsetDateTime::UNIX_EPOCH
}

impl ProductStore for FakeStore {
    async fn create_product(
        &self,
        _draft: &ProductDraft,
        _source_code: Option<&str>,
    ) -> Result<Product> {
        Err(unused("create_product"))
    }

    async fn get_product(&self, _id: Uuid) -> Result<Product> {
        Err(unused("get_product"))
    }

    async fn list_products(&self) -> Result<Vec<Product>> {
        Ok(self.state().products.clone())
    }

    async fn list_products_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Product>> {
        let products = self
            .state()
            .products
            .iter()
            .filter(|product| ids.contains(&product.id))
            .cloned()
            .collect();
        Ok(products)
    }

    async fn update_product(&self, _id: Uuid, _draft: &ProductDraft) -> Result<Product> {
        Err(unused("update_product"))
    }

    async fn delete_product(&self, _id: Uuid) -> Result<()> {
        Err(unused("delete_product"))
    }
}

impl MealStore for FakeStore {
    async fn create_meal(&self, _draft: &MealDraft) -> Result<Meal> {
        Err(unused("create_meal"))
    }

    async fn get_meal(&self, _id: Uuid) -> Result<Meal> {
        Err(unused("get_meal"))
    }

    async fn list_meals(&self) -> Result<Vec<Meal>> {
        Ok(self.state().meals.clone())
    }

    async fn list_meals_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Meal>> {
        let meals = self
            .state()
            .meals
            .iter()
            .filter(|meal| ids.contains(&meal.id))
            .cloned()
            .collect();
        Ok(meals)
    }

    async fn update_meal(&self, _id: Uuid, _draft: &MealDraft) -> Result<Meal> {
        Err(unused("update_meal"))
    }

    async fn delete_meal(&self, _id: Uuid) -> Result<()> {
        Err(unused("delete_meal"))
    }

    async fn meal_label_taken(
        &self,
        _label: &str,
        _category_id: Option<Uuid>,
        _exclude: Option<Uuid>,
    ) -> Result<bool> {
        Err(unused("meal_label_taken"))
    }
}

impl CategoryStore for FakeStore {
    async fn create_category(&self, _draft: &CategoryDraft) -> Result<Category> {
        Err(unused("create_category"))
    }

    async fn get_category(&self, _id: Uuid) -> Result<Category> {
        Err(unused("get_category"))
    }

    async fn list_categories(&self, _scope: CategoryScope) -> Result<Vec<Category>> {
        Err(unused("list_categories"))
    }

    async fn rename_category(&self, _id: Uuid, _name: &str) -> Result<Category> {
        Err(unused("rename_category"))
    }

    async fn delete_category(&self, _id: Uuid) -> Result<()> {
        Err(unused("delete_category"))
    }

    async fn category_name_taken(
        &self,
        _name: &str,
        _scope: CategoryScope,
        _exclude: Option<Uuid>,
    ) -> Result<bool> {
        Err(unused("category_name_taken"))
    }
}

impl DayPlanStore for FakeStore {
    async fn create_day_plan(&self, _draft: &DayPlanDraft) -> Result<DayPlan> {
        Err(unused("create_day_plan"))
    }

    async fn create_day_plan_copy(
        &self,
        source_plan_id: Uuid,
        draft: &DayPlanDraft,
    ) -> Result<DayPlan> {
        let mut state = self.state();
        let source = state
            .plans
            .iter()
            .find(|plan| plan.id == source_plan_id)
            .ok_or_else(|| Error::NotFound {
                entity: Entity::DayPlan,
                id: source_plan_id.to_string(),
            })?;
        // A copy of a copy records the original, so the chain never grows.
        let source_plan_id = source.source_plan_id.unwrap_or(source.id);
        let copy = DayPlan {
            id: Uuid::new_v4(),
            label: draft.label.clone(),
            category_id: draft.category_id,
            source_plan_id: Some(source_plan_id),
            items: draft.items.clone(),
            created_at: now(),
            updated_at: now(),
        };
        state.plans.push(copy.clone());
        Ok(copy)
    }

    async fn get_day_plan(&self, id: Uuid) -> Result<DayPlan> {
        self.state()
            .plans
            .iter()
            .find(|plan| plan.id == id)
            .cloned()
            .ok_or_else(|| Error::NotFound {
                entity: Entity::DayPlan,
                id: id.to_string(),
            })
    }

    async fn list_day_plans(&self) -> Result<Vec<DayPlan>> {
        let mut plans: Vec<DayPlan> = self
            .state()
            .plans
            .iter()
            .filter(|plan| plan.source_plan_id.is_none())
            .cloned()
            .collect();
        plans.sort_by(|left, right| left.label.cmp(&right.label));
        Ok(plans)
    }

    async fn list_day_plans_by_ids(&self, ids: &[Uuid]) -> Result<Vec<DayPlan>> {
        let plans = self
            .state()
            .plans
            .iter()
            .filter(|plan| ids.contains(&plan.id))
            .cloned()
            .collect();
        Ok(plans)
    }

    async fn update_day_plan(&self, id: Uuid, draft: &DayPlanDraft) -> Result<DayPlan> {
        let mut state = self.state();
        let plan = state
            .plans
            .iter_mut()
            .find(|plan| plan.id == id)
            .ok_or_else(|| Error::NotFound {
                entity: Entity::DayPlan,
                id: id.to_string(),
            })?;
        plan.label = draft.label.clone();
        plan.category_id = draft.category_id;
        plan.items = draft.items.clone();
        Ok(plan.clone())
    }

    async fn delete_day_plan(&self, _id: Uuid) -> Result<()> {
        Err(unused("delete_day_plan"))
    }

    async fn day_plan_label_taken(
        &self,
        _label: &str,
        _category_id: Option<Uuid>,
        _exclude: Option<Uuid>,
    ) -> Result<bool> {
        Err(unused("day_plan_label_taken"))
    }
}

impl CalendarStore for FakeStore {
    async fn assign_calendar_day(&self, date: Date, day_plan_id: Uuid) -> Result<CalendarDay> {
        self.state().calendar.insert(date, day_plan_id);
        Ok(CalendarDay {
            date,
            day_plan_id,
            created_at: now(),
            updated_at: now(),
        })
    }

    async fn list_calendar_days(&self, from: Date, to: Date) -> Result<Vec<CalendarDay>> {
        let days = self
            .state()
            .calendar
            .range(from..=to)
            .map(|(date, day_plan_id)| CalendarDay {
                date: *date,
                day_plan_id: *day_plan_id,
                created_at: now(),
                updated_at: now(),
            })
            .collect();
        Ok(days)
    }

    async fn unassign_calendar_day(&self, date: Date) -> Result<()> {
        self.state()
            .calendar
            .remove(&date)
            .map(|_| ())
            .ok_or_else(|| Error::NotFound {
                entity: Entity::CalendarDay,
                id: date.to_string(),
            })
    }
}

impl TargetsStore for FakeStore {
    async fn get_targets(&self) -> Result<Option<Targets>> {
        Ok(self.state().targets.clone())
    }

    async fn set_targets(&self, macros: Macros) -> Result<Targets> {
        let targets = Targets {
            macros,
            created_at: now(),
            updated_at: now(),
        };
        self.state().targets = Some(targets.clone());
        Ok(targets)
    }
}

impl AssistantStore for FakeStore {
    async fn get_assistant_key(&self) -> Result<Option<String>> {
        Ok(self.state().key.clone())
    }

    async fn set_assistant_key(&self, key: &str) -> Result<()> {
        self.state().key = Some(key.to_owned());
        Ok(())
    }

    async fn clear_assistant_key(&self) -> Result<()> {
        self.state().key = None;
        Ok(())
    }

    async fn record_assistant_request(
        &self,
        draft: &AssistantRecordDraft,
    ) -> Result<AssistantRecord> {
        let record = AssistantRecord {
            id: Uuid::new_v4(),
            request: draft.request.clone(),
            rationale: draft.rationale.clone(),
            operations: draft.operations.clone(),
            changes: draft.changes.clone(),
            created_at: now(),
        };
        self.state().log.insert(0, record.clone());
        Ok(record)
    }

    async fn list_assistant_requests(&self, limit: u32) -> Result<Vec<AssistantRecord>> {
        let limit = usize::try_from(limit).unwrap_or(usize::MAX);
        Ok(self.state().log.iter().take(limit).cloned().collect())
    }
}

// the fixture

/// One plan shared by eight consecutive dates, half of them already past: the shape the
/// whole copy-on-write question is about.
struct Fixture {
    whey: Uuid,
    shake: Uuid,
    training: Uuid,
    today: Date,
}

struct Harness {
    rules: Mealprep<FakeStore, (), CannedAssistant>,
    fixture: Fixture,
    seen: Seen,
}

fn seed(store: &FakeStore) -> Fixture {
    let today = OffsetDateTime::now_utc().date();

    let whey = Product {
        id: Uuid::new_v4(),
        name: "Whey".to_owned(),
        unit: Unit::Gram,
        macros: Macros {
            energy_kcal: 380.0,
            fat_g: 7.0,
            protein_g: 75.0,
            carbohydrates_g: 8.0,
        },
        nutrients: Nutrients::new(),
        cost: Some(2.0),
        brand: None,
        source_code: None,
        created_at: now(),
        updated_at: now(),
    };
    let shake = Meal {
        id: Uuid::new_v4(),
        label: "Shake".to_owned(),
        category_id: None,
        servings: Vec::new(),
        created_at: now(),
        updated_at: now(),
    };
    let training = DayPlan {
        id: Uuid::new_v4(),
        label: "Training day".to_owned(),
        category_id: None,
        source_plan_id: None,
        items: vec![DayPlanItem::Product {
            product_id: whey.id,
            amount: 60.0,
        }],
        created_at: now(),
        updated_at: now(),
    };

    let mut state = store.state();
    state.products.push(whey.clone());
    state.meals.push(shake.clone());
    state.plans.push(training.clone());
    for offset in -4..4 {
        state
            .calendar
            .insert(today + Duration::days(offset), training.id);
    }
    state.targets = Some(Targets {
        macros: Macros {
            energy_kcal: 2000.0,
            fat_g: 65.0,
            protein_g: 140.0,
            carbohydrates_g: 220.0,
        },
        created_at: now(),
        updated_at: now(),
    });
    state.key = Some("sk-ant-fake-key".to_owned());
    drop(state);

    Fixture {
        whey: whey.id,
        shake: shake.id,
        training: training.id,
        today,
    }
}

/// The rules over a seeded fake store, with the assistant primed to answer whatever the
/// given closure makes of the seeded ids.
fn harness(operations: impl FnOnce(&Fixture) -> Vec<Operation>) -> Harness {
    let store = FakeStore::default();
    let fixture = seed(&store);
    let seen = Seen::default();
    let assistant = CannedAssistant {
        operations: operations(&fixture),
        seen: seen.clone(),
    };
    Harness {
        rules: Mealprep::new(store, (), assistant),
        fixture,
        seen,
    }
}

/// How much of the given product a resolved plan carries as an item of its own.
fn loose_amount(plan: &ResolvedDayPlan, product_id: Uuid) -> Option<f64> {
    plan.items.iter().find_map(|item| match item {
        ResolvedDayPlanItem::Product { product } if product.id == product_id => {
            Some(product.amount)
        }
        _ => None,
    })
}

fn proposal(request: &str, operations: Vec<Operation>) -> Proposal {
    Proposal {
        request: request.to_owned(),
        rationale: "Adjusted the days you named.".to_owned(),
        operations,
    }
}

// the tests

#[tokio::test]
async fn proposes_a_preview_without_writing_anything() {
    let harness = harness(|fixture| {
        vec![Operation::AdjustProductAmount {
            dates: DateRange {
                from: fixture.today,
                to: fixture.today + Duration::days(3),
            },
            product_id: fixture.whey,
            delta_amount: -40.0,
        }]
    });
    let (rules, fixture) = (&harness.rules, &harness.fixture);

    let request = "next 4 days I have 40 g less whey".to_owned();
    let preview = rules.propose_assistant_edit(request.clone()).await.unwrap();

    // The request is the rules' own, never the one the assistant answered with.
    assert_eq!(preview.proposal.request, request);
    assert_eq!(preview.days.len(), 4);
    assert_eq!(preview.days[0].date, fixture.today);
    assert!(preview.days.iter().all(|day| day.detaches));
    assert!(preview.days.iter().all(|day| day.date >= fixture.today));

    // 60 g of whey down to 20 g, at 380 kcal per 100 g.
    let day = &preview.days[0];
    assert!((day.before.energy_kcal - 0.6 * 380.0).abs() < 1e-9);
    assert!((day.after.energy_kcal - 0.2 * 380.0).abs() < 1e-9);
    assert!(
        (day.target_delta.unwrap().energy_kcal - (0.2 * 380.0 - 2000.0)).abs() < 1e-9,
        "the delta reads against the target"
    );
    assert_eq!(day.changes, vec!["Whey 60 g → 20 g".to_owned()]);

    // The assistant was handed the stored key, the request as typed, and a context that
    // names what it is allowed to touch.
    {
        let seen = harness.seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].0, "sk-ant-fake-key");
        assert_eq!(seen[0].1, request);
        assert_eq!(seen[0].2.today, fixture.today);
        assert!(
            seen[0]
                .2
                .plans
                .iter()
                .any(|plan| plan.id == fixture.training),
            "the plan is assignable"
        );
    }

    // Nothing was written: the plan still has its 60 g and the log is empty.
    let plan = rules.get_day_plan(fixture.training).await.unwrap();
    assert_eq!(loose_amount(&plan, fixture.whey), Some(60.0));
    assert!(rules.list_assistant_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn applying_detaches_only_the_dates_it_names() {
    let harness = harness(|_| Vec::new());
    let (rules, fixture) = (&harness.rules, &harness.fixture);

    let operations = vec![Operation::AdjustProductAmount {
        dates: DateRange {
            from: fixture.today,
            to: fixture.today + Duration::days(3),
        },
        product_id: fixture.whey,
        delta_amount: -40.0,
    }];
    let record = rules
        .apply_assistant_edit(proposal(
            "next 4 days I have 40 g less whey",
            operations.clone(),
        ))
        .await
        .unwrap();

    assert_eq!(record.changes.len(), 4);
    for change in &record.changes {
        assert!(change.detached);
        assert_eq!(change.previous_day_plan_id, Some(fixture.training));
        assert_eq!(change.previous_label.as_deref(), Some("Training day"));

        // The date now points at its own copy, not at the plan it shared.
        let copy_id = change.day_plan_id.unwrap();
        assert_ne!(copy_id, fixture.training);
        let copy = rules.get_day_plan(copy_id).await.unwrap();
        assert_eq!(copy.source_plan_id, Some(fixture.training));
        assert_eq!(copy.label, "Training day");
        assert_eq!(loose_amount(&copy, fixture.whey), Some(20.0));
    }

    // Every past date still points at the untouched original, so history cannot move.
    let past = rules
        .list_calendar_days(
            fixture.today - Duration::days(4),
            fixture.today - Duration::days(1),
        )
        .await
        .unwrap();
    assert_eq!(past.len(), 4);
    assert!(past.iter().all(|day| day.day_plan_id == fixture.training));
    let original = rules.get_day_plan(fixture.training).await.unwrap();
    assert_eq!(loose_amount(&original, fixture.whey), Some(60.0));

    // The copies are not plans the user keeps, so they stay out of the list.
    let kept = rules.list_day_plans().await.unwrap();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].id, fixture.training);

    // The log carries what was asked, what was applied, and what it replaced.
    let log = rules.list_assistant_requests().await.unwrap();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].id, record.id);
    assert_eq!(log[0].request, "next 4 days I have 40 g less whey");
    assert_eq!(log[0].operations, operations);
}

#[tokio::test]
async fn a_second_edit_to_a_detached_date_reuses_its_copy() {
    let harness = harness(|_| Vec::new());
    let (rules, fixture) = (&harness.rules, &harness.fixture);
    let dates = DateRange::just(fixture.today);

    let first = rules
        .apply_assistant_edit(proposal(
            "40 g less whey today",
            vec![Operation::AdjustProductAmount {
                dates,
                product_id: fixture.whey,
                delta_amount: -40.0,
            }],
        ))
        .await
        .unwrap();
    let copy_id = first.changes[0].day_plan_id.unwrap();

    let second = rules
        .apply_assistant_edit(proposal(
            "add the shake today",
            vec![Operation::AddItem {
                dates,
                item: DayPlanItem::Meal {
                    meal_id: fixture.shake,
                },
            }],
        ))
        .await
        .unwrap();

    // The date already has a copy, so it is edited in place and no second one is made.
    assert!(!second.changes[0].detached);
    assert_eq!(second.changes[0].day_plan_id, Some(copy_id));
    assert_eq!(second.changes[0].previous_day_plan_id, Some(copy_id));

    let edited = rules.get_day_plan(copy_id).await.unwrap();
    assert_eq!(edited.items.len(), 2);
    assert_eq!(edited.source_plan_id, Some(fixture.training));
    assert_eq!(rules.list_day_plans().await.unwrap().len(), 1);
}

#[tokio::test]
async fn refuses_a_past_date_and_writes_nothing_when_one_operation_fails() {
    let harness = harness(|_| Vec::new());
    let (rules, fixture) = (&harness.rules, &harness.fixture);

    let past = rules
        .apply_assistant_edit(proposal(
            "40 g less whey",
            vec![Operation::AdjustProductAmount {
                dates: DateRange::just(fixture.today - Duration::days(1)),
                product_id: fixture.whey,
                delta_amount: -40.0,
            }],
        ))
        .await;
    assert!(matches!(past, Err(Error::Invalid { .. })));

    // A workable operation followed by an impossible one: the whole request is refused,
    // so the first date is never written either.
    let mixed = rules
        .apply_assistant_edit(proposal(
            "less whey today, and drop the shake tomorrow",
            vec![
                Operation::AdjustProductAmount {
                    dates: DateRange::just(fixture.today),
                    product_id: fixture.whey,
                    delta_amount: -40.0,
                },
                Operation::RemoveItem {
                    dates: DateRange::just(fixture.today + Duration::days(1)),
                    item: ItemRef::Meal {
                        meal_id: fixture.shake,
                    },
                },
            ],
        ))
        .await;
    assert!(matches!(mixed, Err(Error::Invalid { .. })));

    assert_eq!(rules.list_day_plans().await.unwrap().len(), 1);
    let plan = rules.get_day_plan(fixture.training).await.unwrap();
    assert_eq!(loose_amount(&plan, fixture.whey), Some(60.0));
    assert!(rules.list_assistant_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn without_a_key_the_assistant_is_off() {
    let harness = harness(|_| Vec::new());
    let rules = &harness.rules;
    rules.clear_assistant_key().await.unwrap();

    let status = rules.assistant_key_status().await.unwrap();
    assert!(!status.present && status.hint.is_none());
    assert!(matches!(
        rules.propose_assistant_edit("anything".to_owned()).await,
        Err(Error::Invalid { .. })
    ));
    assert!(harness.seen.lock().unwrap().is_empty(), "nothing was sent");

    let status = rules
        .set_assistant_key("  sk-ant-api03-tail  ".to_owned())
        .await
        .unwrap();
    assert!(status.present);
    assert_eq!(status.hint.as_deref(), Some("tail"));
    assert!(
        rules
            .set_assistant_key("not a key".to_owned())
            .await
            .is_err(),
        "a key with a space in it is a paste that went wrong"
    );
}

#[tokio::test]
async fn a_detached_date_is_still_joined_to_its_plan_on_the_calendar() {
    let harness = harness(|_| Vec::new());
    let (rules, fixture) = (&harness.rules, &harness.fixture);

    rules
        .apply_assistant_edit(proposal(
            "40 g less whey today",
            vec![Operation::AdjustProductAmount {
                dates: DateRange::just(fixture.today),
                product_id: fixture.whey,
                delta_amount: -40.0,
            }],
        ))
        .await
        .unwrap();

    // The plan list cannot resolve that date any more, so the calendar read has to.
    let plans = rules
        .list_calendar_day_plans(fixture.today, fixture.today + Duration::days(1))
        .await
        .unwrap();
    let detached = plans
        .iter()
        .find(|plan| plan.source_plan_id == Some(fixture.training))
        .expect("the copy is among the plans the dates are assigned");
    assert!((detached.macros.energy_kcal - 0.2 * 380.0).abs() < 1e-9);
}
