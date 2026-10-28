//! Exercises the business rules against a real, in-memory SQLite store.

use std::sync::{Arc, Mutex};

use mealprep_core::{
    Entity, Error, Result,
    domain::{
        AssistantContext, CatalogueEntry, CategoryDraft, CategoryScope, DateRange, DayPlanDraft,
        DayPlanItem, EntrySource, Macros, MealDraft, Nutrients, Operation, ProductDraft, Proposal,
        ResolvedDayPlan, ResolvedDayPlanItem, Serving, Unit, parse_iso_date,
    },
    service::Mealprep,
    store::{Assistant, Catalogue},
};
use mealprep_sqlite::SqliteStore;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

// test types

#[derive(Default)]
struct FixedCatalogue {
    entries: Vec<CatalogueEntry>,
}

impl Catalogue for FixedCatalogue {
    async fn search_catalogue(&self, query: &str) -> Result<Vec<CatalogueEntry>> {
        let entries = self
            .entries
            .iter()
            .filter(|entry| entry.name.contains(query))
            .cloned()
            .collect();
        Ok(entries)
    }

    async fn get_catalogue_entry(&self, code: &str) -> Result<CatalogueEntry> {
        self.entries
            .iter()
            .find(|entry| entry.code == code)
            .cloned()
            .ok_or_else(|| Error::NotFound {
                entity: Entity::CatalogueEntry,
                id: code.to_owned(),
            })
    }
}

/// Stands in for the model: the test writes the operations, the rules check them. What
/// a real adapter would add is an HTTP call, which these tests are not about.
#[derive(Clone, Default)]
struct ScriptedAssistant {
    answer: Arc<Mutex<Vec<Operation>>>,
}

impl ScriptedAssistant {
    fn answers(&self, operations: Vec<Operation>) {
        *self.answer.lock().unwrap() = operations;
    }
}

impl Assistant for ScriptedAssistant {
    async fn propose(
        &self,
        _key: &str,
        request: &str,
        _context: &AssistantContext,
    ) -> Result<Proposal> {
        Ok(Proposal {
            request: request.to_owned(),
            rationale: "Adjusted the days you named.".to_owned(),
            operations: self.answer.lock().unwrap().clone(),
        })
    }
}

type Rules = Mealprep<SqliteStore, FixedCatalogue, ScriptedAssistant>;

async fn mealprep(catalogue: FixedCatalogue) -> Rules {
    let store = SqliteStore::open_in_memory().await.unwrap();
    Mealprep::new(store, catalogue, ScriptedAssistant::default())
}

/// The rules and the handle that scripts what the assistant will answer.
async fn with_assistant() -> (Rules, ScriptedAssistant) {
    let store = SqliteStore::open_in_memory().await.unwrap();
    let assistant = ScriptedAssistant::default();
    let rules = Mealprep::new(store, FixedCatalogue::default(), assistant.clone());
    (rules, assistant)
}

fn product_draft(name: &str, energy_kcal: f64, cost: Option<f64>) -> ProductDraft {
    ProductDraft {
        name: name.to_owned(),
        unit: Unit::Gram,
        macros: Macros {
            energy_kcal,
            fat_g: 1.0,
            protein_g: 2.0,
            carbohydrates_g: 3.0,
        },
        nutrients: [("fiber".to_owned(), 4.0)].into(),
        cost,
        brand: None,
    }
}

fn entry(code: &str) -> CatalogueEntry {
    CatalogueEntry {
        code: code.to_owned(),
        name: "Skyr".to_owned(),
        unit: Unit::Gram,
        macros: Macros {
            energy_kcal: 63.0,
            fat_g: 0.2,
            protein_g: 11.0,
            carbohydrates_g: 4.0,
        },
        nutrients: Nutrients::new(),
        brand: Some("arla".to_owned()),
        image_url: None,
        missing_macros: Vec::new(),
        source: EntrySource::Community,
        warnings: Vec::new(),
        modified_at: None,
    }
}

#[tokio::test]
async fn products_are_created_updated_and_soft_deleted() {
    let mealprep = mealprep(FixedCatalogue::default()).await;

    let created = mealprep
        .create_product(product_draft(" Oats ", 380.0, None))
        .await
        .unwrap();
    assert_eq!(created.name, "Oats");
    assert_eq!(created.cost, None);
    assert_eq!(created.nutrients.get("fiber"), Some(&4.0));

    let updated = mealprep
        .update_product(created.id, product_draft("Rolled oats", 370.0, Some(0.4)))
        .await
        .unwrap();
    assert_eq!(updated.cost, Some(0.4));
    assert_eq!(updated.created_at, created.created_at);

    mealprep.delete_product(created.id).await.unwrap();
    assert!(mealprep.list_products().await.unwrap().is_empty());
    assert!(matches!(
        mealprep.get_product(created.id).await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        mealprep.delete_product(created.id).await,
        Err(Error::NotFound { .. })
    ));
}

#[tokio::test]
async fn catalogue_entries_are_fetched_for_review_and_imported_as_reviewed() {
    let catalogue = FixedCatalogue {
        entries: vec![entry("123")],
    };
    let mealprep = mealprep(catalogue).await;

    let found = mealprep.search_catalogue("Sky").await.unwrap();
    assert_eq!(found.len(), 1);
    assert!(matches!(
        mealprep.search_catalogue("   ").await,
        Err(Error::Invalid { .. })
    ));

    let fetched = mealprep.get_catalogue_entry(" 123 ").await.unwrap();
    assert_eq!(fetched.code, "123");
    assert!(matches!(
        mealprep.get_catalogue_entry("789").await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        mealprep.get_catalogue_entry("12/3").await,
        Err(Error::Invalid { .. })
    ));

    // The user corrected the catalogue's energy against the package before importing.
    let reviewed = ProductDraft {
        brand: Some("arla".to_owned()),
        ..product_draft("Skyr", 64.0, None)
    };
    let imported = mealprep.import_product("123", reviewed).await.unwrap();
    assert_eq!(imported.source_code.as_deref(), Some("123"));
    assert_eq!(imported.macros.energy_kcal, 64.0);
    assert_eq!(imported.brand.as_deref(), Some("arla"));

    assert!(matches!(
        mealprep
            .import_product("", product_draft("Skyr", 64.0, None))
            .await,
        Err(Error::Invalid { .. })
    ));
}

#[tokio::test]
async fn meals_derive_figures_and_keep_labels_unique_per_category() {
    let mealprep = mealprep(FixedCatalogue::default()).await;
    let oats = mealprep
        .create_product(product_draft("Oats", 400.0, Some(1.0)))
        .await
        .unwrap();
    let milk = mealprep
        .create_product(product_draft("Milk", 60.0, None))
        .await
        .unwrap();
    let breakfasts = mealprep
        .create_category(CategoryDraft {
            name: "Breakfasts".to_owned(),
            scope: CategoryScope::Meal,
        })
        .await
        .unwrap();

    let draft = MealDraft {
        label: "Porridge".to_owned(),
        category_id: Some(breakfasts.id),
        servings: vec![
            Serving {
                product_id: milk.id,
                amount: 200.0,
            },
            Serving {
                product_id: oats.id,
                amount: 50.0,
            },
        ],
    };
    let meal = mealprep.create_meal(draft.clone()).await.unwrap();
    assert_eq!(meal.macros.energy_kcal, 320.0);
    assert!(!meal.cost.complete);
    assert_eq!(meal.cost.amount, 0.5);
    assert_eq!(meal.entity.servings, draft.servings);

    let shouting = MealDraft {
        label: "PORRIDGE".to_owned(),
        ..draft.clone()
    };
    assert!(matches!(
        mealprep.create_meal(shouting).await,
        Err(Error::Conflict { .. })
    ));

    let uncategorised = MealDraft {
        category_id: None,
        ..draft.clone()
    };
    let other = mealprep.create_meal(uncategorised.clone()).await.unwrap();
    assert!(matches!(
        mealprep.create_meal(uncategorised).await,
        Err(Error::Conflict { .. })
    ));

    let reordered = MealDraft {
        servings: draft.servings.iter().rev().copied().collect(),
        ..draft
    };
    let updated = mealprep
        .update_meal(meal.entity.id, reordered.clone())
        .await
        .unwrap();
    assert_eq!(updated.entity.servings, reordered.servings);
    assert_eq!(
        mealprep
            .get_meal(meal.entity.id)
            .await
            .unwrap()
            .entity
            .servings,
        reordered.servings
    );

    let labels: Vec<_> = mealprep
        .list_meals()
        .await
        .unwrap()
        .into_iter()
        .map(|meal| meal.entity.id)
        .collect();
    assert!(labels.contains(&other.entity.id));
}

#[tokio::test]
async fn compositions_refuse_foreign_categories_and_missing_references() {
    let mealprep = mealprep(FixedCatalogue::default()).await;
    let plans_category = mealprep
        .create_category(CategoryDraft {
            name: "Cutting".to_owned(),
            scope: CategoryScope::DayPlan,
        })
        .await
        .unwrap();

    let in_wrong_list = MealDraft {
        label: "Lunch".to_owned(),
        category_id: Some(plans_category.id),
        servings: Vec::new(),
    };
    assert!(matches!(
        mealprep.create_meal(in_wrong_list).await,
        Err(Error::Invalid { .. })
    ));

    let missing_product = MealDraft {
        label: "Lunch".to_owned(),
        category_id: None,
        servings: vec![Serving {
            product_id: Uuid::new_v4(),
            amount: 10.0,
        }],
    };
    assert!(matches!(
        mealprep.create_meal(missing_product).await,
        Err(Error::Invalid { .. })
    ));

    let missing_meal = DayPlanDraft {
        label: "Rest day".to_owned(),
        category_id: None,
        items: vec![DayPlanItem::Meal {
            meal_id: Uuid::new_v4(),
        }],
    };
    assert!(matches!(
        mealprep.create_day_plan(missing_meal).await,
        Err(Error::Invalid { .. })
    ));
}

#[tokio::test]
async fn categories_are_unique_within_their_scope_and_keep_it_on_rename() {
    let mealprep = mealprep(FixedCatalogue::default()).await;
    let meal_category = mealprep
        .create_category(CategoryDraft {
            name: "Quick".to_owned(),
            scope: CategoryScope::Meal,
        })
        .await
        .unwrap();
    let plan_category = mealprep
        .create_category(CategoryDraft {
            name: "quick".to_owned(),
            scope: CategoryScope::DayPlan,
        })
        .await
        .unwrap();

    assert!(matches!(
        mealprep
            .create_category(CategoryDraft {
                name: "QUICK".to_owned(),
                scope: CategoryScope::Meal,
            })
            .await,
        Err(Error::Conflict { .. })
    ));

    let renamed = mealprep
        .rename_category(plan_category.id, "Slow")
        .await
        .unwrap();
    assert_eq!(renamed.scope, CategoryScope::DayPlan);
    assert!(matches!(
        mealprep.rename_category(renamed.id, "  ").await,
        Err(Error::Invalid { .. })
    ));

    mealprep.delete_category(meal_category.id).await.unwrap();
    assert!(
        mealprep
            .list_categories(CategoryScope::Meal)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn day_plans_resolve_items_in_order_and_survive_deleted_references() {
    let mealprep = mealprep(FixedCatalogue::default()).await;
    let oats = mealprep
        .create_product(product_draft("Oats", 400.0, Some(1.0)))
        .await
        .unwrap();
    let apple = mealprep
        .create_product(product_draft("Apple", 50.0, Some(0.2)))
        .await
        .unwrap();
    let breakfast = mealprep
        .create_meal(MealDraft {
            label: "Breakfast".to_owned(),
            category_id: None,
            servings: vec![Serving {
                product_id: oats.id,
                amount: 100.0,
            }],
        })
        .await
        .unwrap();

    let plan = mealprep
        .create_day_plan(DayPlanDraft {
            label: "Rest day".to_owned(),
            category_id: None,
            items: vec![
                DayPlanItem::Product {
                    product_id: apple.id,
                    amount: 200.0,
                },
                DayPlanItem::Meal {
                    meal_id: breakfast.entity.id,
                },
            ],
        })
        .await
        .unwrap();
    assert_eq!(plan.macros.energy_kcal, 500.0);
    assert_eq!(plan.cost.amount, 1.4);
    assert!(plan.cost.complete);
    assert!(matches!(
        &plan.items[0],
        ResolvedDayPlanItem::Product { product } if product.name.as_deref() == Some("Apple")
    ));

    mealprep.delete_product(apple.id).await.unwrap();
    mealprep.delete_meal(breakfast.entity.id).await.unwrap();

    let after = mealprep.get_day_plan(plan.id).await.unwrap();
    assert_eq!(after.items.len(), 1);
    assert!(matches!(
        &after.items[0],
        ResolvedDayPlanItem::Product { product } if product.name.is_none() && product.amount == 200.0
    ));
    assert!(!after.cost.complete);
    assert_eq!(after.macros, Macros::ZERO);

    let plans = mealprep.list_day_plans().await.unwrap();
    assert_eq!(plans.len(), 1);
}

#[tokio::test]
async fn calendar_days_hold_one_plan_each() {
    let mealprep = mealprep(FixedCatalogue::default()).await;
    let first = mealprep
        .create_day_plan(DayPlanDraft {
            label: "Rest day".to_owned(),
            category_id: None,
            items: Vec::new(),
        })
        .await
        .unwrap();
    let second = mealprep
        .create_day_plan(DayPlanDraft {
            label: "Training day".to_owned(),
            category_id: None,
            items: Vec::new(),
        })
        .await
        .unwrap();
    let monday = parse_iso_date("2026-09-07").unwrap();
    let tuesday = parse_iso_date("2026-09-08").unwrap();
    let sunday = parse_iso_date("2026-09-13").unwrap();

    mealprep
        .assign_calendar_day(monday, first.id)
        .await
        .unwrap();
    mealprep
        .assign_calendar_day(tuesday, first.id)
        .await
        .unwrap();
    let replaced = mealprep
        .assign_calendar_day(monday, second.id)
        .await
        .unwrap();
    assert_eq!(replaced.day_plan_id, second.id);

    let week = mealprep.list_calendar_days(monday, sunday).await.unwrap();
    assert_eq!(week.len(), 2);
    assert_eq!(week[0].date, monday);

    mealprep.unassign_calendar_day(tuesday).await.unwrap();
    assert!(matches!(
        mealprep.unassign_calendar_day(tuesday).await,
        Err(Error::NotFound { .. })
    ));
    mealprep
        .assign_calendar_day(tuesday, first.id)
        .await
        .unwrap();

    assert!(matches!(
        mealprep.assign_calendar_day(sunday, Uuid::new_v4()).await,
        Err(Error::Invalid { .. })
    ));
    assert!(matches!(
        mealprep.list_calendar_days(sunday, monday).await,
        Err(Error::Invalid { .. })
    ));
    let far = parse_iso_date("2028-01-01").unwrap();
    assert!(matches!(
        mealprep.list_calendar_days(monday, far).await,
        Err(Error::Invalid { .. })
    ));
}

#[tokio::test]
async fn targets_start_unset_and_refuse_negative_figures() {
    let mealprep = mealprep(FixedCatalogue::default()).await;
    assert!(mealprep.get_targets().await.unwrap().is_none());

    let macros = Macros {
        energy_kcal: 2400.0,
        fat_g: 80.0,
        protein_g: 180.0,
        carbohydrates_g: 240.0,
    };
    mealprep.set_targets(macros).await.unwrap();
    let raised = Macros {
        energy_kcal: 2600.0,
        ..macros
    };
    mealprep.set_targets(raised).await.unwrap();
    assert_eq!(
        mealprep.get_targets().await.unwrap().unwrap().macros,
        raised
    );

    let negative = Macros {
        protein_g: -1.0,
        ..macros
    };
    assert!(matches!(
        mealprep.set_targets(negative).await,
        Err(Error::Invalid { .. })
    ));
}

#[tokio::test]
async fn weigh_ins_replace_by_date_and_smooth_into_a_trend() {
    let mealprep = mealprep(FixedCatalogue::default()).await;
    assert!(mealprep.weight_series().await.unwrap().points.is_empty());

    let monday = parse_iso_date("2026-09-07").unwrap();
    let tuesday = parse_iso_date("2026-09-08").unwrap();
    mealprep.set_weight_entry(monday, 80.0).await.unwrap();
    mealprep.set_weight_entry(tuesday, 90.0).await.unwrap();

    // A second weigh-in for a date replaces the first rather than adding a row.
    mealprep.set_weight_entry(tuesday, 81.0).await.unwrap();
    assert_eq!(mealprep.list_weight_entries().await.unwrap().len(), 2);

    let series = mealprep.weight_series().await.unwrap();
    assert_eq!(series.points.len(), 2);
    let trend = series.trend.unwrap();
    assert!((trend - 80.1).abs() < 1e-9, "trend was {trend}");

    // The history reads most recent first.
    let entries = mealprep.list_weight_entries().await.unwrap();
    assert_eq!(entries[0].date, tuesday);

    // An implausible figure is refused, and a date without a weigh-in cannot be cleared.
    assert!(matches!(
        mealprep.set_weight_entry(monday, 8000.0).await,
        Err(Error::Invalid { .. })
    ));
    let never_logged = parse_iso_date("2026-01-01").unwrap();
    assert!(matches!(
        mealprep.delete_weight_entry(never_logged).await,
        Err(Error::NotFound { .. })
    ));

    mealprep.delete_weight_entry(tuesday).await.unwrap();
    assert_eq!(mealprep.list_weight_entries().await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_database_file_keeps_its_rows_across_openings() {
    let path = std::env::temp_dir().join(format!("mealprep-test-{}.db", Uuid::new_v4()));

    let first = SqliteStore::open(&path).await.unwrap();
    let mealprep = Mealprep::new(
        first,
        FixedCatalogue::default(),
        ScriptedAssistant::default(),
    );
    mealprep
        .create_product(product_draft("Oats", 380.0, None))
        .await
        .unwrap();
    drop(mealprep);

    let second = SqliteStore::open(&path).await.unwrap();
    let mealprep = Mealprep::new(
        second,
        FixedCatalogue::default(),
        ScriptedAssistant::default(),
    );
    assert_eq!(mealprep.list_products().await.unwrap().len(), 1);
    drop(mealprep);

    for suffix in ["", "-wal", "-shm"] {
        let file = format!("{}{suffix}", path.display());
        std::fs::remove_file(file).ok();
    }
}

#[tokio::test]
async fn an_assistant_edit_detaches_the_dates_it_names_and_leaves_the_past_alone() {
    let (mealprep, assistant) = with_assistant().await;
    let whey = mealprep
        .create_product(product_draft("Whey", 380.0, Some(2.0)))
        .await
        .unwrap();
    let plan = mealprep
        .create_day_plan(DayPlanDraft {
            label: "Training day".to_owned(),
            category_id: None,
            items: vec![DayPlanItem::Product {
                product_id: whey.id,
                amount: 60.0,
            }],
        })
        .await
        .unwrap();

    // The same plan on two days gone and two to come, which is the case the whole
    // copy-on-write question is about.
    let today = OffsetDateTime::now_utc().date();
    for offset in [-2, -1, 0, 1] {
        mealprep
            .assign_calendar_day(today + Duration::days(offset), plan.id)
            .await
            .unwrap();
    }

    mealprep
        .set_assistant_key("sk-ant-test-key".to_owned())
        .await
        .unwrap();
    assistant.answers(vec![Operation::AdjustProductAmount {
        dates: DateRange {
            from: today,
            to: today + Duration::days(1),
        },
        product_id: whey.id,
        delta_amount: -40.0,
    }]);

    let preview = mealprep
        .propose_assistant_edit("next 2 days I have 40 g less whey".to_owned())
        .await
        .unwrap();
    assert_eq!(preview.days.len(), 2);
    assert!(preview.days.iter().all(|day| day.detaches));

    // Proposing wrote nothing.
    assert_eq!(mealprep.list_day_plans().await.unwrap().len(), 1);

    let record = mealprep
        .apply_assistant_edit(preview.proposal.clone())
        .await
        .unwrap();
    assert_eq!(record.changes.len(), 2);

    // The copies are not plans the user keeps, so neither the list nor a picker sees them.
    let kept = mealprep.list_day_plans().await.unwrap();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].id, plan.id);
    assert_eq!(amount_of(&kept[0], whey.id), Some(60.0));

    // Both past dates still point at the original, so the history cannot have moved.
    for offset in [-2, -1] {
        let date = today + Duration::days(offset);
        let days = mealprep.list_calendar_days(date, date).await.unwrap();
        assert_eq!(days[0].day_plan_id, plan.id, "{date} moved");
    }

    // The two future dates point at copies of it, one each, carrying the new amount.
    let mut copies = Vec::new();
    for offset in [0, 1] {
        let date = today + Duration::days(offset);
        let days = mealprep.list_calendar_days(date, date).await.unwrap();
        let assigned = days[0].day_plan_id;
        assert_ne!(assigned, plan.id);
        let copy = mealprep.get_day_plan(assigned).await.unwrap();
        assert_eq!(copy.source_plan_id, Some(plan.id));
        assert_eq!(copy.label, "Training day");
        assert_eq!(amount_of(&copy, whey.id), Some(20.0));
        copies.push(assigned);
    }
    assert_ne!(copies[0], copies[1], "each date gets its own copy");

    // A view that joins dates to plans can still resolve a detached date.
    let joined = mealprep
        .list_calendar_day_plans(today - Duration::days(2), today + Duration::days(1))
        .await
        .unwrap();
    assert_eq!(joined.len(), 3, "the original and the two copies");

    // A copy shares the label it was copied from, and that is not a clash with it.
    mealprep
        .update_day_plan(
            plan.id,
            DayPlanDraft {
                label: "Training day".to_owned(),
                category_id: None,
                items: vec![DayPlanItem::Product {
                    product_id: whey.id,
                    amount: 60.0,
                }],
            },
        )
        .await
        .unwrap();

    // The log survives the round trip through the database, operations included.
    let log = mealprep.list_assistant_requests().await.unwrap();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].id, record.id);
    assert_eq!(log[0].request, "next 2 days I have 40 g less whey");
    assert_eq!(log[0].operations, preview.proposal.operations);
    assert_eq!(log[0].changes, record.changes);
    assert!(
        log[0]
            .changes
            .iter()
            .all(|change| change.previous_day_plan_id == Some(plan.id) && change.detached)
    );

    // A second edit to a date that is already detached reuses its copy.
    assistant.answers(vec![Operation::AddItem {
        dates: DateRange::just(today),
        item: DayPlanItem::Product {
            product_id: whey.id,
            amount: 10.0,
        },
    }]);
    let preview = mealprep
        .propose_assistant_edit("make it another 10 g today".to_owned())
        .await
        .unwrap();
    assert!(
        !preview.days[0].detaches,
        "the date already has its own copy"
    );
    let record = mealprep
        .apply_assistant_edit(preview.proposal)
        .await
        .unwrap();
    assert_eq!(record.changes[0].day_plan_id, Some(copies[0]));
    assert_eq!(mealprep.list_day_plans().await.unwrap().len(), 1);
    assert_eq!(mealprep.list_assistant_requests().await.unwrap().len(), 2);
}

#[tokio::test]
async fn the_assistant_refuses_a_past_date_unless_the_request_names_one() {
    let (mealprep, assistant) = with_assistant().await;
    let whey = mealprep
        .create_product(product_draft("Whey", 380.0, Some(2.0)))
        .await
        .unwrap();
    let plan = mealprep
        .create_day_plan(DayPlanDraft {
            label: "Training day".to_owned(),
            category_id: None,
            items: vec![DayPlanItem::Product {
                product_id: whey.id,
                amount: 60.0,
            }],
        })
        .await
        .unwrap();

    let today = OffsetDateTime::now_utc().date();
    let yesterday = today - Duration::days(1);
    mealprep
        .assign_calendar_day(yesterday, plan.id)
        .await
        .unwrap();
    mealprep
        .set_assistant_key("sk-ant-test-key".to_owned())
        .await
        .unwrap();
    assistant.answers(vec![Operation::AdjustProductAmount {
        dates: DateRange::just(yesterday),
        product_id: whey.id,
        delta_amount: -40.0,
    }]);

    assert!(matches!(
        mealprep
            .propose_assistant_edit("40 g less whey".to_owned())
            .await,
        Err(Error::Invalid { .. })
    ));

    // The same operation, from a request that says the past date out loud, goes through.
    let preview = mealprep
        .propose_assistant_edit("yesterday I had 40 g less whey".to_owned())
        .await
        .unwrap();
    mealprep
        .apply_assistant_edit(preview.proposal)
        .await
        .unwrap();
    let assigned = mealprep
        .list_calendar_days(yesterday, yesterday)
        .await
        .unwrap()[0]
        .day_plan_id;
    assert_ne!(assigned, plan.id, "the past date got its own copy too");
    assert_eq!(
        amount_of(&mealprep.get_day_plan(plan.id).await.unwrap(), whey.id),
        Some(60.0)
    );
}

#[tokio::test]
async fn the_assistant_is_off_until_a_key_is_pasted_in() {
    let mealprep = mealprep(FixedCatalogue::default()).await;

    let status = mealprep.assistant_key_status().await.unwrap();
    assert!(!status.present && status.hint.is_none());
    assert!(matches!(
        mealprep
            .propose_assistant_edit("anything at all".to_owned())
            .await,
        Err(Error::Invalid { .. })
    ));

    let status = mealprep
        .set_assistant_key("sk-ant-api03-tail".to_owned())
        .await
        .unwrap();
    assert!(status.present);
    // The key is never handed back in full, only the tail of it.
    assert_eq!(status.hint.as_deref(), Some("tail"));

    mealprep.clear_assistant_key().await.unwrap();
    assert!(!mealprep.assistant_key_status().await.unwrap().present);
    // Clearing again is not a failure: the end state is what was asked for.
    mealprep.clear_assistant_key().await.unwrap();
}

#[tokio::test]
async fn a_sync_writes_the_catalogues_figures_and_links_the_product_to_its_code() {
    let catalogue = FixedCatalogue {
        entries: vec![entry("5701234567890")],
    };
    let mealprep = mealprep(catalogue).await;
    mealprep
        .set_assistant_key("sk-ant-test-key".to_owned())
        .await
        .unwrap();
    let skyr = mealprep
        .create_product(product_draft("My skyr", 60.0, Some(0.5)))
        .await
        .unwrap();
    assert_eq!(skyr.source_code, None);

    mealprep
        .apply_assistant_edit(Proposal {
            request: "sync my skyr, barcode 5701234567890".to_owned(),
            rationale: "Synced your skyr.".to_owned(),
            operations: vec![Operation::SyncProduct {
                product_id: skyr.id,
                code: Some("5701234567890".to_owned()),
            }],
        })
        .await
        .unwrap();

    let synced = mealprep.get_product(skyr.id).await.unwrap();
    assert_eq!(synced.source_code.as_deref(), Some("5701234567890"));
    assert_eq!(synced.name, "My skyr");
    assert_eq!(synced.cost, Some(0.5));
    assert_eq!(synced.macros.energy_kcal, 63.0);
    assert_eq!(synced.brand.as_deref(), Some("arla"));
    // The entry states no fibre, so the product no longer claims any.
    assert!(synced.nutrients.is_empty(), "{:?}", synced.nutrients);
}

/// How much of the given product a resolved plan carries as an item of its own.
fn amount_of(plan: &ResolvedDayPlan, product_id: Uuid) -> Option<f64> {
    plan.items.iter().find_map(|item| match item {
        ResolvedDayPlanItem::Product { product } if product.id == product_id => {
            Some(product.amount)
        }
        _ => None,
    })
}
