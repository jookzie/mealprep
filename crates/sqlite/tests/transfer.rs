//! Exporting the database as one file and checking a file before it is imported.
//!
//! An import replaces the only copy of the user's data, and a file the application cannot
//! open stops it starting, so what the check refuses matters as much as what it accepts.

use std::{fs, path::PathBuf};

use mealprep_core::{
    Entity, Error,
    domain::{Macros, ProductDraft, Unit},
    store::ProductStore,
};
use mealprep_sqlite::{SqliteStore, prepare_import};

fn scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("mealprep-transfer-{}-{name}", std::process::id()));
    fs::remove_dir_all(&directory).ok();
    fs::create_dir_all(&directory).expect("create the scratch directory");
    directory
}

fn demo_database() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../demo/mealprep-demo.db")
}

async fn run(path: &PathBuf, sql: &'static str) {
    let store = SqliteStore::open(path).await.expect("open");
    drop(store);
    let options = sqlx::sqlite::SqliteConnectOptions::new().filename(path);
    let mut connection = <sqlx::SqliteConnection as sqlx::Connection>::connect_with(&options)
        .await
        .expect("connect");
    sqlx::raw_sql(sql)
        .execute(&mut connection)
        .await
        .expect("run");
}

fn assert_refused(result: mealprep_core::Result<()>, needle: &str) {
    match result {
        Err(Error::Invalid {
            entity: Entity::Database,
            reason,
        }) => assert!(
            reason.contains(needle),
            "{reason:?} should mention {needle:?}"
        ),
        other => panic!("expected the import to be refused, got {other:?}"),
    }
}

#[tokio::test]
async fn a_snapshot_is_importable_and_holds_the_data() {
    let directory = scratch("snapshot");
    let source = directory.join("mealprep.db");
    fs::copy(demo_database(), &source).unwrap();
    let store = SqliteStore::open(&source).await.unwrap();
    let products = store.list_products().await.unwrap().len();

    let snapshot = directory.join("export.db");
    store.snapshot_to(&snapshot).await.expect("snapshot");

    prepare_import(&snapshot)
        .await
        .expect("a snapshot is importable");
    let copy = SqliteStore::open(&snapshot).await.unwrap();
    assert_eq!(copy.list_products().await.unwrap().len(), products);

    fs::remove_dir_all(directory).ok();
}

#[tokio::test]
async fn a_database_from_an_older_build_is_importable() {
    // The demo database predates the latest migration, as a file from an older install does.
    let directory = scratch("older");
    let path = directory.join("mealprep.db");
    fs::copy(demo_database(), &path).unwrap();
    prepare_import(&path)
        .await
        .expect("an older database is importable");
    fs::remove_dir_all(directory).ok();
}

#[tokio::test]
async fn a_file_copied_with_its_log_keeps_the_latest_writes() {
    // What a copy off the phone looks like: the store is still open, so the newest rows
    // live in the write-ahead log beside the file rather than in the file itself.
    let directory = scratch("wal");
    let live = directory.join("mealprep.db");
    fs::copy(demo_database(), &live).unwrap();
    let store = SqliteStore::open(&live).await.unwrap();
    let draft = ProductDraft {
        name: "Written last".to_owned(),
        unit: Unit::Gram,
        macros: Macros {
            energy_kcal: 100.0,
            fat_g: 1.0,
            protein_g: 1.0,
            carbohydrates_g: 1.0,
        },
        nutrients: Default::default(),
        cost: None,
        brand: None,
    };
    store.create_product(&draft, None).await.unwrap();
    let expected = store.list_products().await.unwrap().len();

    let copied = directory.join("copy");
    fs::create_dir_all(&copied).unwrap();
    for suffix in ["", "-wal", "-shm"] {
        let from = directory.join(format!("mealprep.db{suffix}"));
        if from.exists() {
            fs::copy(&from, copied.join(format!("mealprep.db{suffix}"))).unwrap();
        }
    }
    assert!(
        copied.join("mealprep.db-wal").exists(),
        "the test needs a log to fold in"
    );

    let path = copied.join("mealprep.db");
    prepare_import(&path).await.expect("importable");
    assert!(
        !copied.join("mealprep.db-wal").exists(),
        "the log is folded into the file"
    );

    // Moved on its own, as the import moves it, the file still holds the last write.
    let moved = directory.join("moved.db");
    fs::rename(&path, &moved).unwrap();
    let reopened = SqliteStore::open(&moved).await.unwrap();
    let products = reopened.list_products().await.unwrap();
    assert_eq!(products.len(), expected);
    assert!(
        products
            .iter()
            .any(|product| product.name == "Written last")
    );

    drop(store);
    fs::remove_dir_all(directory).ok();
}

#[tokio::test]
async fn a_database_from_a_newer_build_is_refused() {
    let directory = scratch("newer");
    let path = directory.join("mealprep.db");
    run(
        &path,
        "INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) \
         VALUES (9999, 'from the future', CURRENT_TIMESTAMP, 1, x'00', 0)",
    )
    .await;

    assert_refused(prepare_import(&path).await, "newer version");
    fs::remove_dir_all(directory).ok();
}

#[tokio::test]
async fn a_database_with_an_edited_migration_is_refused() {
    let directory = scratch("edited");
    let path = directory.join("mealprep.db");
    run(
        &path,
        "UPDATE _sqlx_migrations SET checksum = x'00' WHERE version = 1",
    )
    .await;

    assert_refused(prepare_import(&path).await, "differs");
    fs::remove_dir_all(directory).ok();
}

#[tokio::test]
async fn a_file_that_is_not_a_mealprep_database_is_refused() {
    let directory = scratch("foreign");

    let text = directory.join("notes.db");
    fs::write(
        &text,
        "not a database at all, just some text that is long enough",
    )
    .unwrap();
    assert_refused(prepare_import(&text).await, "SQLite");

    let other = directory.join("other.db");
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&other)
        .create_if_missing(true);
    let mut connection = <sqlx::SqliteConnection as sqlx::Connection>::connect_with(&options)
        .await
        .unwrap();
    sqlx::raw_sql("CREATE TABLE things (id INTEGER)")
        .execute(&mut connection)
        .await
        .unwrap();
    drop(connection);
    assert_refused(prepare_import(&other).await, "not a Mealprep database");

    assert_refused(
        prepare_import(&directory.join("missing.db")).await,
        "SQLite",
    );
    fs::remove_dir_all(directory).ok();
}
