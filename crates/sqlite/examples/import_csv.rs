//! One-off import of the spreadsheet export into a fresh database.
//!
//! Takes the normalised JSON (products, meals) rather than the CSV itself, so the
//! spreadsheet's quirks — unit suffixes, a euro sign, blocks of empty padding rows —
//! are dealt with before anything reaches the rules. Everything is written through
//! `Mealprep`, so a row that the rules would reject fails here rather than landing in
//! the database unvalidated.
//!
//! Usage: `cargo run -p mealprep-sqlite --example import_csv -- <products.json> <meals.json> <out.db>`

use std::{collections::HashMap, env, fs, process};

use mealprep_core::{
    Entity, Error, Result,
    domain::{CatalogueEntry, Macros, MealDraft, ProductDraft, Serving, Unit},
    service::Mealprep,
    store::Catalogue,
};
use mealprep_sqlite::SqliteStore;
use serde_json::Value;

/// The import needs no catalogue, but `Mealprep` is generic over one.
struct NoCatalogue;

impl Catalogue for NoCatalogue {
    async fn search_catalogue(&self, _query: &str) -> Result<Vec<CatalogueEntry>> {
        Ok(Vec::new())
    }

    async fn get_catalogue_entry(&self, code: &str) -> Result<CatalogueEntry> {
        Err(Error::NotFound {
            entity: Entity::CatalogueEntry,
            id: code.to_owned(),
        })
    }
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("import failed: {error}");
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let [products_path, meals_path, db_path] = args.as_slice() else {
        eprintln!("usage: import_csv <products.json> <meals.json> <out.db>");
        process::exit(2);
    };

    let products: Value = read_json(products_path);
    let meals: Value = read_json(meals_path);

    let store = SqliteStore::open(db_path.as_ref()).await?;
    let mealprep = Mealprep::new(store, NoCatalogue);

    let mut ids: HashMap<String, uuid::Uuid> = HashMap::new();
    for entry in products.as_array().expect("products is an array") {
        let name = entry["name"].as_str().expect("name").to_owned();
        let macros = &entry["macros"];
        let draft = ProductDraft {
            name: name.clone(),
            // The export carries no unit column and every serving in it is written in
            // grams, so grams is the only reading that keeps the two consistent.
            unit: Unit::Gram,
            macros: Macros {
                energy_kcal: number(&macros["energy"]),
                fat_g: number(&macros["fat"]),
                protein_g: number(&macros["protein"]),
                carbohydrates_g: number(&macros["carbs"]),
            },
            nutrients: Default::default(),
            cost: entry["cost"].as_f64(),
            brand: None,
        };
        let product = mealprep.create_product(draft).await?;
        ids.insert(name, product.id);
    }
    println!("{} products", ids.len());

    let mut count = 0;
    for (label, servings) in meals.as_object().expect("meals is an object") {
        let servings: Vec<Serving> = servings
            .as_array()
            .expect("servings is an array")
            .iter()
            .map(|serving| {
                let name = serving["product"].as_str().expect("product");
                Serving {
                    product_id: *ids
                        .get(name)
                        .unwrap_or_else(|| panic!("meal {label:?} names unknown product {name:?}")),
                    amount: number(&serving["amount"]),
                }
            })
            .collect();
        mealprep
            .create_meal(MealDraft {
                label: label.clone(),
                category_id: None,
                servings,
            })
            .await?;
        count += 1;
    }
    println!("{count} meals");

    Ok(())
}

fn read_json(path: &str) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|error| panic!("read {path}: {error}"));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {path}: {error}"))
}

fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or_default()
}
