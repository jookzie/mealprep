//! Exercises the business rules against a real, in-memory SQLite store.

use mealprep_core::{
    Entity, Error, Result,
    domain::{
        ActivityDay, CatalogueEntry, CategoryDraft, CategoryScope, DayPlanDraft, DayPlanItem,
        EntrySource, HealthImport, Macros, MealDraft, MeasurementKind, Nutrients, ProductDraft,
        Reading, ResolvedDayPlanItem, Serving, SleepSession, Unit, parse_iso_date,
    },
    service::Mealprep,
    store::Catalogue,
};
use mealprep_sqlite::SqliteStore;
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

async fn mealprep(catalogue: FixedCatalogue) -> Mealprep<SqliteStore, FixedCatalogue> {
    let store = SqliteStore::open_in_memory().await.unwrap();
    Mealprep::new(store, catalogue)
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
async fn measurements_replace_by_date_and_kind() {
    let mealprep = mealprep(FixedCatalogue::default()).await;
    let monday = parse_iso_date("2026-09-07").unwrap();

    mealprep
        .set_measurement(monday, MeasurementKind::Waist, 90.0)
        .await
        .unwrap();
    mealprep
        .set_measurement(monday, MeasurementKind::Height, 180.0)
        .await
        .unwrap();
    // The same kind on the same date corrects the figure rather than adding one.
    mealprep
        .set_measurement(monday, MeasurementKind::Waist, 88.5)
        .await
        .unwrap();

    let measurements = mealprep.list_measurements().await.unwrap();
    assert_eq!(measurements.len(), 2);
    let waist = measurements
        .iter()
        .find(|measurement| measurement.kind == MeasurementKind::Waist)
        .unwrap();
    assert_eq!(waist.value, 88.5);

    assert!(matches!(
        mealprep
            .set_measurement(monday, MeasurementKind::BodyFat, 0.2)
            .await,
        Err(Error::Invalid { .. })
    ));
    assert!(matches!(
        mealprep
            .delete_measurement(monday, MeasurementKind::Hips)
            .await,
        Err(Error::NotFound { .. })
    ));

    mealprep
        .delete_measurement(monday, MeasurementKind::Waist)
        .await
        .unwrap();
    assert_eq!(mealprep.list_measurements().await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_health_import_replaces_the_days_it_covers_and_nothing_else() {
    let mealprep = mealprep(FixedCatalogue::default()).await;
    let today = parse_iso_date("2026-09-10").unwrap();
    let now = time::OffsetDateTime::UNIX_EPOCH;

    // Nothing imported yet: the first import reaches back a year.
    let start = mealprep.health_import_start(today).await.unwrap();
    assert_eq!(start, parse_iso_date("2025-09-10").unwrap());

    let first = HealthImport {
        sleep: vec![SleepSession {
            start: time::macros::datetime!(2026-09-01 23:00 +02:00),
            end: time::macros::datetime!(2026-09-02 07:00 +02:00),
            stages: Vec::new(),
        }],
        heart_rate_variability: vec![Reading {
            at: time::macros::datetime!(2026-09-09 05:00 +02:00),
            value: 60.0,
        }],
        activity: vec![ActivityDay {
            date: parse_iso_date("2026-09-08").unwrap(),
            active_kcal: Some(400.0),
            total_kcal: Some(2600.0),
            exercise_minutes: Some(30.0),
        }],
        ..HealthImport::default()
    };
    mealprep
        .import_health(start, today, first, now)
        .await
        .unwrap();

    let days = mealprep.list_health_days().await.unwrap();
    assert_eq!(days.len(), 3);
    let night = days[0].sleep.expect("the night of the 1st");
    assert_eq!((night.bed_minute, night.wake_minute), (-60, 420));

    // A later import re-reads only the last few days, and a day the source no longer has
    // anything for goes, while older days stay.
    let start = mealprep.health_import_start(today).await.unwrap();
    assert_eq!(start, parse_iso_date("2026-09-07").unwrap());
    mealprep
        .import_health(start, today, HealthImport::default(), now)
        .await
        .unwrap();

    let days = mealprep.list_health_days().await.unwrap();
    assert_eq!(days.len(), 1);
    assert_eq!(days[0].date, parse_iso_date("2026-09-02").unwrap());
    let sync = mealprep.last_health_sync().await.unwrap().unwrap();
    assert_eq!(sync.through, today);

    // The derived reads run over the stored days.
    let sleep = mealprep.sleep_summary(today).await.unwrap();
    assert_eq!(sleep.nights.len(), 1);
    let recovery = mealprep.recovery(today).await.unwrap();
    assert_eq!(recovery.hrv, None);
}

#[tokio::test]
async fn energy_balance_reads_the_plan_and_the_weight_trend() {
    let mealprep = mealprep(FixedCatalogue::default()).await;
    let product = mealprep
        .create_product(product_draft("Oats", 400.0, None))
        .await
        .unwrap();
    let plan = mealprep
        .create_day_plan(DayPlanDraft {
            label: "Steady".to_owned(),
            category_id: None,
            items: vec![DayPlanItem::Product {
                product_id: product.id,
                amount: 500.0,
            }],
        })
        .await
        .unwrap();

    let today = parse_iso_date("2026-09-30").unwrap();
    for offset in 1..=21 {
        let date = today - time::Duration::days(offset);
        mealprep.assign_calendar_day(date, plan.id).await.unwrap();
    }
    for offset in 0..=30 {
        let date = today - time::Duration::days(offset);
        mealprep.set_weight_entry(date, 80.0).await.unwrap();
    }

    let balance = mealprep.energy_balance(today).await.unwrap();

    assert_eq!(balance.planned_days, 21);
    assert_eq!(balance.planned_kcal, Some(2000.0));
    // A flat trend means everything planned was burned.
    let expenditure = balance.expenditure_kcal.unwrap();
    assert!((expenditure - 2000.0).abs() < 1e-6, "was {expenditure}");
    assert_eq!(balance.measured_kcal, None);
}

#[tokio::test]
async fn a_database_file_keeps_its_rows_across_openings() {
    let path = std::env::temp_dir().join(format!("mealprep-test-{}.db", Uuid::new_v4()));

    let first = SqliteStore::open(&path).await.unwrap();
    let mealprep = Mealprep::new(first, FixedCatalogue::default());
    mealprep
        .create_product(product_draft("Oats", 380.0, None))
        .await
        .unwrap();
    drop(mealprep);

    let second = SqliteStore::open(&path).await.unwrap();
    let mealprep = Mealprep::new(second, FixedCatalogue::default());
    assert_eq!(mealprep.list_products().await.unwrap().len(), 1);
    drop(mealprep);

    for suffix in ["", "-wal", "-shm"] {
        let file = format!("{}{suffix}", path.display());
        std::fs::remove_file(file).ok();
    }
}
