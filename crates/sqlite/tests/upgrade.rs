//! Opens a database written by an older build and checks that the data survives it.
//!
//! The migrations run whenever the store opens, including on the phone, where the file
//! they run against is the only copy of the user's data and an uninstall is the only way
//! back. A migration that drops or rewrites a table destructively compiles, passes every
//! test that starts from an empty database, and is only noticed once it has run on the
//! device. The checked-in demo dataset is the stand-in for that file: it was written by a
//! build older than this one, so migrating it here is the check that an update can never
//! be the thing that loses the data.
//!
//! Rebuilding `demo/mealprep-demo.db` with `mise run demo:seed` refreshes the dataset but
//! also resets its age, which is why the file is checked in rather than generated: the
//! point is to migrate something this build did not write.

use std::{fs, path::PathBuf};

use mealprep_core::domain::{CategoryScope, parse_iso_date};
use mealprep_core::store::{
    CalendarStore, CategoryStore, DayPlanStore, MealStore, ProductStore, TargetsStore, WeightStore,
};
use mealprep_sqlite::SqliteStore;

/// The dataset `docs/demo-data.md` describes, which the acceptance build also ships.
fn demo_database() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../demo/mealprep-demo.db")
}

/// Copies the dataset somewhere writable, because opening it runs the migrations and a
/// test must not leave the checked-in file changed.
fn copy_to_scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("mealprep-upgrade-{}-{name}", std::process::id()));
    fs::create_dir_all(&directory).expect("create the scratch directory");
    let path = directory.join("mealprep.db");
    fs::copy(demo_database(), &path).expect("copy the demo database");
    path
}

#[tokio::test]
async fn the_demo_database_still_reads_after_the_migrations_run() {
    let path = copy_to_scratch("reads");

    let store = SqliteStore::open(&path).await.expect("migrate and open");

    // One assertion per table the migrations could touch: a destructive migration usually
    // empties one of them rather than failing the open.
    assert!(!store.list_products().await.unwrap().is_empty(), "products");
    assert!(!store.list_meals().await.unwrap().is_empty(), "meals");
    assert!(
        !store.list_day_plans().await.unwrap().is_empty(),
        "day plans"
    );
    assert!(
        !store
            .list_categories(CategoryScope::Meal)
            .await
            .unwrap()
            .is_empty(),
        "meal categories"
    );
    assert!(
        !store
            .list_categories(CategoryScope::DayPlan)
            .await
            .unwrap()
            .is_empty(),
        "day plan categories"
    );
    assert!(
        !store
            .list_calendar_days(
                parse_iso_date("2000-01-01").unwrap(),
                parse_iso_date("2100-01-01").unwrap()
            )
            .await
            .unwrap()
            .is_empty(),
        "calendar days"
    );
    assert!(
        !store.list_weight_entries().await.unwrap().is_empty(),
        "weigh-ins"
    );
    assert!(store.get_targets().await.unwrap().is_some(), "targets");

    // The composed read, because an entity surviving a migration with its joins broken is
    // the same loss from the user's side.
    let plans = store.list_day_plans().await.unwrap();
    let plan = store.get_day_plan(plans[0].id).await.unwrap();
    assert!(!plan.items.is_empty(), "the first day plan keeps its items");

    fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[tokio::test]
async fn migrating_the_demo_database_twice_changes_nothing() {
    let path = copy_to_scratch("idempotent");

    let first = SqliteStore::open(&path).await.expect("migrate and open");
    let products = first.list_products().await.unwrap().len();
    drop(first);

    // The second open finds every migration applied and must be a no-op. A migration file
    // edited after it shipped fails here rather than on the device.
    let second = SqliteStore::open(&path).await.expect("reopen");
    assert_eq!(second.list_products().await.unwrap().len(), products);

    fs::remove_dir_all(path.parent().unwrap()).ok();
}
