//! Drives the assistant rules end to end with no database and no model.
//!
//! The store is a map in memory, the catalogue a fixed list and the assistant a canned
//! answer, so what is under
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
        AssistantContext, AssistantRecord, AssistantRecordDraft, CalendarDay, CatalogueEntry,
        Category, CategoryDraft, CategoryScope, ChatTurn, DateRange, DayPlan, DayPlanDraft,
        DayPlanItem, EntityAction, EntityKind, EntrySource, ItemRef, MacroField, Macros, Meal,
        MealDraft, MealPatch, Nutrients, Operation, Product, ProductDraft, ProductPatch, Proposal,
        ResolvedDayPlan, ResolvedDayPlanItem, Serving, Targets, TargetsPatch, Unit,
    },
    service::Mealprep,
    store::{
        Assistant, AssistantStore, CalendarStore, Catalogue, CategoryStore, DayPlanStore,
        MealStore, ProductStore, TargetsStore,
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
        _history: &[ChatTurn],
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

/// The catalogue, as one entry: the whey as Open Food Facts has it now.
struct FixedCatalogue;

/// The code the catalogue knows the whey by.
const WHEY_CODE: &str = "5000000000001";

impl Catalogue for FixedCatalogue {
    async fn search_catalogue(&self, _query: &str) -> Result<Vec<CatalogueEntry>> {
        Err(unused("search_catalogue"))
    }

    async fn get_catalogue_entry(&self, code: &str) -> Result<CatalogueEntry> {
        if code != WHEY_CODE {
            return Err(Error::NotFound {
                entity: Entity::CatalogueEntry,
                id: code.to_owned(),
            });
        }
        Ok(CatalogueEntry {
            code: code.to_owned(),
            name: "Whey isolate".to_owned(),
            unit: Unit::Gram,
            // Fat is missing from the entry, so it reads as zero here.
            macros: Macros {
                energy_kcal: 400.0,
                fat_g: 0.0,
                protein_g: 78.0,
                carbohydrates_g: 8.0,
            },
            nutrients: [("calcium".to_owned(), 0.6), ("sodium".to_owned(), 0.2)].into(),
            brand: Some("Acme".to_owned()),
            image_url: None,
            missing_macros: vec![MacroField::FatG],
            source: EntrySource::Manufacturer,
            warnings: Vec::new(),
            modified_at: None,
        })
    }
}

#[derive(Default)]
struct State {
    categories: Vec<Category>,
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
        draft: &ProductDraft,
        source_code: Option<&str>,
    ) -> Result<Product> {
        let product = Product {
            id: Uuid::new_v4(),
            name: draft.name.clone(),
            unit: draft.unit,
            macros: draft.macros,
            nutrients: draft.nutrients.clone(),
            cost: draft.cost,
            brand: draft.brand.clone(),
            source_code: source_code.map(str::to_owned),
            created_at: now(),
            updated_at: now(),
        };
        self.state().products.push(product.clone());
        Ok(product)
    }

    async fn get_product(&self, id: Uuid) -> Result<Product> {
        self.state()
            .products
            .iter()
            .find(|product| product.id == id)
            .cloned()
            .ok_or_else(|| Error::NotFound {
                entity: Entity::Product,
                id: id.to_string(),
            })
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

    async fn update_product(&self, id: Uuid, draft: &ProductDraft) -> Result<Product> {
        let mut state = self.state();
        let product = state
            .products
            .iter_mut()
            .find(|product| product.id == id)
            .ok_or_else(|| Error::NotFound {
                entity: Entity::Product,
                id: id.to_string(),
            })?;
        product.name = draft.name.clone();
        product.unit = draft.unit;
        product.macros = draft.macros;
        product.nutrients = draft.nutrients.clone();
        product.cost = draft.cost;
        product.brand = draft.brand.clone();
        product.updated_at = now();
        Ok(product.clone())
    }

    async fn set_product_source(&self, id: Uuid, source_code: &str) -> Result<()> {
        let mut state = self.state();
        let product = state
            .products
            .iter_mut()
            .find(|product| product.id == id)
            .ok_or_else(|| Error::NotFound {
                entity: Entity::Product,
                id: id.to_string(),
            })?;
        product.source_code = Some(source_code.to_owned());
        Ok(())
    }

    async fn delete_product(&self, _id: Uuid) -> Result<()> {
        Err(unused("delete_product"))
    }
}

impl MealStore for FakeStore {
    async fn create_meal(&self, draft: &MealDraft) -> Result<Meal> {
        let meal = Meal {
            id: Uuid::new_v4(),
            label: draft.label.clone(),
            category_id: draft.category_id,
            servings: draft.servings.clone(),
            created_at: now(),
            updated_at: now(),
        };
        self.state().meals.push(meal.clone());
        Ok(meal)
    }

    async fn get_meal(&self, id: Uuid) -> Result<Meal> {
        self.state()
            .meals
            .iter()
            .find(|meal| meal.id == id)
            .cloned()
            .ok_or_else(|| Error::NotFound {
                entity: Entity::Meal,
                id: id.to_string(),
            })
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

    async fn update_meal(&self, id: Uuid, draft: &MealDraft) -> Result<Meal> {
        let mut state = self.state();
        let meal = state
            .meals
            .iter_mut()
            .find(|meal| meal.id == id)
            .ok_or_else(|| Error::NotFound {
                entity: Entity::Meal,
                id: id.to_string(),
            })?;
        meal.label = draft.label.clone();
        meal.category_id = draft.category_id;
        meal.servings = draft.servings.clone();
        meal.updated_at = now();
        Ok(meal.clone())
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

    async fn list_categories(&self, scope: CategoryScope) -> Result<Vec<Category>> {
        let categories = self
            .state()
            .categories
            .iter()
            .filter(|category| category.scope == scope)
            .cloned()
            .collect();
        Ok(categories)
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
    async fn create_day_plan(&self, draft: &DayPlanDraft) -> Result<DayPlan> {
        let plan = DayPlan {
            id: Uuid::new_v4(),
            label: draft.label.clone(),
            category_id: draft.category_id,
            source_plan_id: None,
            items: draft.items.clone(),
            created_at: now(),
            updated_at: now(),
        };
        self.state().plans.push(plan.clone());
        Ok(plan)
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
            entity_changes: draft.entity_changes.clone(),
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
    breakfast: Uuid,
    today: Date,
}

struct Harness {
    rules: Mealprep<FakeStore, FixedCatalogue, CannedAssistant>,
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
        nutrients: [("calcium".to_owned(), 0.5)].into(),
        cost: Some(2.0),
        brand: None,
        source_code: None,
        created_at: now(),
        updated_at: now(),
    };
    let breakfast = Category {
        id: Uuid::new_v4(),
        name: "Breakfast".to_owned(),
        scope: CategoryScope::Meal,
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
    state.categories.push(breakfast.clone());
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
        breakfast: breakfast.id,
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
        rules: Mealprep::new(store, FixedCatalogue, assistant),
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
    let preview = rules
        .propose_assistant_edit(request.clone(), Vec::new())
        .await
        .unwrap();

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
async fn applying_writes_new_things_with_their_real_ids_and_puts_them_on_dates() {
    let harness = harness(|_| Vec::new());
    let (rules, fixture) = (&harness.rules, &harness.fixture);
    let (oats, bowl, plan) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());

    let operations = vec![
        Operation::CreateProduct {
            new_id: oats,
            product: ProductDraft {
                name: "Oats".to_owned(),
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
                        product_id: fixture.whey,
                        amount: 30.0,
                    },
                ],
            },
        },
        Operation::CreateDayPlan {
            new_id: plan,
            day_plan: DayPlanDraft {
                label: "Oat day".to_owned(),
                category_id: None,
                items: vec![DayPlanItem::Meal { meal_id: bowl }],
            },
        },
        Operation::AssignPlan {
            dates: DateRange::just(fixture.today),
            day_plan_id: plan,
        },
        Operation::UpdateProduct {
            product_id: fixture.whey,
            changes: ProductPatch {
                cost: Some(Some(2.5)),
                ..ProductPatch::default()
            },
        },
    ];
    let record = rules
        .apply_assistant_edit(proposal(
            "an oat day today; whey costs 2.50 now",
            operations,
        ))
        .await
        .unwrap();

    let written: Vec<_> = record
        .entity_changes
        .iter()
        .map(|change| (change.kind, change.action, change.label.as_str()))
        .collect();
    assert_eq!(
        written,
        vec![
            (EntityKind::Product, EntityAction::Create, "Oats"),
            (EntityKind::Product, EntityAction::Update, "Whey"),
            (EntityKind::Meal, EntityAction::Create, "Oat bowl"),
            (EntityKind::DayPlan, EntityAction::Create, "Oat day"),
        ]
    );
    // The store's ids, not the ones the model picked.
    let real = |kind| {
        record
            .entity_changes
            .iter()
            .find(|change| change.kind == kind && change.action == EntityAction::Create)
            .unwrap()
            .id
    };
    let (real_oats, real_bowl, real_plan) = (
        real(EntityKind::Product),
        real(EntityKind::Meal),
        real(EntityKind::DayPlan),
    );
    assert!(real_oats != oats && real_bowl != bowl && real_plan != plan);

    // Each new thing names the others by their real ids, and today is on the new plan.
    let meal = rules.get_meal(real_bowl).await.unwrap();
    assert_eq!(meal.entity.servings[0].product_id, real_oats);
    let day_plan = rules.get_day_plan(real_plan).await.unwrap();
    assert_eq!(day_plan.label, "Oat day");
    let today = rules
        .list_calendar_days(fixture.today, fixture.today)
        .await
        .unwrap();
    assert_eq!(today[0].day_plan_id, real_plan);
    assert_eq!(record.changes[0].day_plan_id, Some(real_plan));

    // The update kept every field it did not name.
    let whey = rules.get_product(fixture.whey).await.unwrap();
    assert_eq!(whey.cost, Some(2.5));
    assert!((whey.macros.protein_g - 75.0).abs() < 1e-9);

    // And the log keeps what was created, for the history.
    let log = rules.list_assistant_requests().await.unwrap();
    assert_eq!(log[0].entity_changes.len(), 4);
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
async fn syncing_takes_the_catalogues_figures_and_links_a_hand_entered_product() {
    let harness = harness(|_| Vec::new());
    let (rules, fixture) = (&harness.rules, &harness.fixture);
    let sync = |code: Option<&str>| {
        proposal(
            "sync my whey from open food facts",
            vec![Operation::SyncProduct {
                product_id: fixture.whey,
                code: code.map(str::to_owned),
            }],
        )
    };

    // Entered by hand, so there is nothing to sync from until a barcode is given.
    let Err(Error::Invalid { reason, .. }) = rules.apply_assistant_edit(sync(None)).await else {
        panic!("a hand-entered product has no code to sync from");
    };
    assert!(reason.contains("barcode"), "{reason}");

    let record = rules
        .apply_assistant_edit(sync(Some(WHEY_CODE)))
        .await
        .unwrap();
    let lines = &record.entity_changes[0].changes;
    assert!(
        lines.iter().any(|line| line.contains(WHEY_CODE)),
        "{lines:?}"
    );
    assert!(lines.contains(&"Brand Acme".to_owned()), "{lines:?}");
    assert!(
        lines.contains(&"Energy per 100 g 380 kcal → 400 kcal".to_owned()),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"Calcium per 100 g 500 mg → 600 mg".to_owned()),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"Sodium per 100 g 200 mg".to_owned()),
        "{lines:?}"
    );

    let whey = rules.get_product(fixture.whey).await.unwrap();
    assert_eq!(whey.source_code.as_deref(), Some(WHEY_CODE));
    assert_eq!(whey.name, "Whey", "the name is the user's and stays");
    assert_eq!(whey.cost, Some(2.0), "so is the price");
    assert_eq!(whey.macros.protein_g, 78.0);
    assert_eq!(
        whey.macros.fat_g, 7.0,
        "a figure the catalogue lacks is kept"
    );
    assert_eq!(whey.nutrients["sodium"], 0.2);

    // Linked now, so the next sync needs no code, and finds nothing left to change.
    let Err(Error::Invalid { reason, .. }) = rules.apply_assistant_edit(sync(None)).await else {
        panic!("a product that matches the catalogue has nothing to sync");
    };
    assert!(reason.contains("already matches"), "{reason}");
}

/// One edit to every kind of field an update can now reach, and a day that shows the
/// targets moving.
fn every_field(fixture: &Fixture) -> Vec<Operation> {
    vec![
        Operation::UpdateProduct {
            product_id: fixture.whey,
            changes: ProductPatch {
                brand: Some(Some("Acme".to_owned())),
                cost: Some(None),
                nutrients: [
                    ("iron".to_owned(), Some(0.002)),
                    ("calcium".to_owned(), None),
                ]
                .into(),
                ..ProductPatch::default()
            },
        },
        Operation::UpdateMeal {
            meal_id: fixture.shake,
            changes: MealPatch {
                category_id: Some(Some(fixture.breakfast)),
                ..MealPatch::default()
            },
        },
        Operation::UpdateTargets {
            changes: TargetsPatch {
                energy_kcal: Some(2200.0),
                ..TargetsPatch::default()
            },
        },
        Operation::AddItem {
            dates: DateRange::just(fixture.today),
            item: DayPlanItem::Meal {
                meal_id: fixture.shake,
            },
        },
    ]
}

#[tokio::test]
async fn an_update_reaches_every_field_and_null_clears_one() {
    let harness = harness(every_field);
    let (rules, fixture) = (&harness.rules, &harness.fixture);

    let preview = rules
        .propose_assistant_edit("acme whey, no price, 2200 kcal".to_owned(), Vec::new())
        .await
        .unwrap();
    let lines: Vec<&str> = preview
        .entities
        .iter()
        .flat_map(|entity| entity.changes.iter().map(String::as_str))
        .collect();
    for expected in [
        "Brand Acme",
        "Cost removed",
        "Iron per 100 g 2 mg",
        "Calcium removed",
        "Filed under Breakfast",
        "Energy a day 2000 kcal → 2200 kcal",
    ] {
        assert!(lines.contains(&expected), "{expected:?} in {lines:?}");
    }
    let targets = preview
        .entities
        .iter()
        .find(|entity| entity.kind == EntityKind::Targets)
        .unwrap();
    assert_eq!(targets.label, "Daily targets");
    // The day reads against the targets the same proposal sets.
    assert_eq!(preview.targets.unwrap().energy_kcal, 2200.0);
    let day = &preview.days[0];
    assert!(
        (day.target_delta.unwrap().energy_kcal - (day.after.energy_kcal - 2200.0)).abs() < 1e-9
    );

    rules
        .apply_assistant_edit(proposal(
            "acme whey, no price, 2200 kcal",
            every_field(fixture),
        ))
        .await
        .unwrap();
    let whey = rules.get_product(fixture.whey).await.unwrap();
    assert_eq!(whey.brand.as_deref(), Some("Acme"));
    assert_eq!(whey.cost, None);
    assert_eq!(whey.nutrients, [("iron".to_owned(), 0.002)].into());
    let shake = rules.get_meal(fixture.shake).await.unwrap();
    assert_eq!(shake.entity.category_id, Some(fixture.breakfast));
    let targets = rules.get_targets().await.unwrap().unwrap();
    assert_eq!(targets.macros.energy_kcal, 2200.0);
    assert_eq!(targets.macros.protein_g, 140.0, "a figure left out stays");
}

#[tokio::test]
async fn a_question_is_answered_in_prose_from_a_context_that_carries_the_nutrients() {
    let harness = harness(|_| Vec::new());
    let (rules, fixture) = (&harness.rules, &harness.fixture);

    let preview = rules
        .propose_assistant_edit("how much calcium am I getting today".to_owned(), Vec::new())
        .await
        .unwrap();
    assert!(preview.entities.is_empty() && preview.days.is_empty());
    assert!(!preview.proposal.rationale.is_empty());

    let seen = harness.seen.lock().unwrap();
    let context = &seen[0].2;
    let today = context
        .days
        .iter()
        .find(|day| day.date == fixture.today)
        .unwrap();
    // 60 g of whey at 0.5 g of calcium per 100 g.
    assert!((today.nutrients["calcium"] - 0.3).abs() < 1e-9, "{today:?}");
    let whey = &context.products[0];
    assert_eq!(whey.nutrients["calcium"], 0.5);
    assert!(!whey.synced, "entered by hand");
    assert_eq!(context.categories[0].name, "Breakfast");
}

#[tokio::test]
async fn without_a_key_the_assistant_is_off() {
    let harness = harness(|_| Vec::new());
    let rules = &harness.rules;
    rules.clear_assistant_key().await.unwrap();

    let status = rules.assistant_key_status().await.unwrap();
    assert!(!status.present && status.hint.is_none());
    assert!(matches!(
        rules
            .propose_assistant_edit("anything".to_owned(), Vec::new())
            .await,
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
