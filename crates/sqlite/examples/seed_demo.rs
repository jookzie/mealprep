//! Builds the demo database: a small, believable dataset that puts every feature on screen.
//!
//! Everything is written through `Mealprep`, so the file is exactly what the app itself
//! would have produced and every rule applies to it. The states that only arise over time
//! — a product deleted out from under a meal, a category removed, a plan unassigned by
//! deletion — are reached the same way the user would reach them: by creating the entity,
//! using it, then deleting it.
//!
//! The calendar, the weigh-ins, the tape measurements and the imported nights are laid out
//! around a reference day, today by default, because the calendar opens on the current week
//! and every insight is read as of "now". `docs/demo-data.md` maps each feature to where the
//! data shows it.
//!
//! Usage: `cargo run -p mealprep-sqlite --example seed_demo -- <out.db> [YYYY-MM-DD]`

use std::{env, fs, path::PathBuf, process};

use mealprep_core::{
    Entity, Error, Result,
    domain::{
        ActivityDay, CatalogueEntry, CategoryDraft, CategoryScope, DayPlanDraft, DayPlanItem,
        HealthImport, Macros, MealDraft, MeasurementKind, Nutrients, ProductDraft, Reading,
        Serving, SleepSession, SleepStage, SleepStageKind, Unit, parse_iso_date,
    },
    service::Mealprep,
    store::Catalogue,
};
use mealprep_sqlite::SqliteStore;
use sqlx::{Connection, SqliteConnection};
use time::{Date, Duration, OffsetDateTime, Weekday, macros::offset};
use uuid::Uuid;

/// The seed needs no catalogue, but `Mealprep` is generic over one. Imports are
/// recorded through `import_product` with the code alone, as a reviewed import stores.
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

type Demo = Mealprep<SqliteStore, NoCatalogue>;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (out, today) = match args.as_slice() {
        [out] => (out, OffsetDateTime::now_utc().date()),
        [out, today] => match parse_iso_date(today) {
            Ok(today) => (out, today),
            Err(error) => {
                eprintln!("seed_demo: {error}");
                process::exit(2);
            }
        },
        _ => {
            eprintln!("usage: seed_demo <out.db> [YYYY-MM-DD]");
            process::exit(2);
        }
    };

    if let Err(error) = run(out.into(), today).await {
        eprintln!("seed failed: {error}");
        process::exit(1);
    }
}

async fn run(out: PathBuf, today: Date) -> Result<()> {
    // Seeding twice into one file would duplicate every entity, so the output has to be new.
    if out.exists() {
        return Err(internal(format!("{} already exists", out.display())));
    }

    // Seeded into a scratch file and copied out with VACUUM INTO, so what is written is one
    // compact file with no WAL sidecars to lose on the way to a device.
    let scratch = env::temp_dir().join(format!("mealprep-seed-{}.db", Uuid::new_v4()));
    let demo = Mealprep::new(SqliteStore::open(&scratch).await?, NoCatalogue);
    seed(&demo, today).await?;
    drop(demo);

    let mut connection = SqliteConnection::connect(&format!("sqlite://{}", scratch.display()))
        .await
        .map_err(|error| internal(error.to_string()))?;
    sqlx::query("VACUUM INTO ?")
        .bind(out.to_string_lossy().into_owned())
        .execute(&mut connection)
        .await
        .map_err(|error| internal(error.to_string()))?;
    connection
        .close()
        .await
        .map_err(|error| internal(error.to_string()))?;
    for suffix in ["", "-wal", "-shm"] {
        let _ = fs::remove_file(format!("{}{suffix}", scratch.display()));
    }

    println!("seeded {} around {today}", out.display());
    Ok(())
}

async fn seed(demo: &Demo, today: Date) -> Result<()> {
    let products = products(demo).await?;
    let categories = categories(demo).await?;
    let meals = meals(demo, &products, &categories).await?;
    let plans = day_plans(demo, &products, &categories, &meals).await?;
    calendar(demo, &plans, today).await?;
    weight(demo, today).await?;
    measurements(demo, today).await?;
    health(demo, today).await?;
    demo.set_targets(macros(2000.0, 65.0, 140.0, 220.0)).await?;

    // Deleted last, once something refers to each, so every join has a miss to render.
    demo.delete_product(products.protein_bar).await?;
    demo.delete_category(categories.seasonal).await?;
    demo.delete_meal(meals.old_smoothie).await?;
    demo.delete_day_plan(plans.old_cut).await?;
    Ok(())
}

// products

struct Products {
    oats: Uuid,
    milk: Uuid,
    banana: Uuid,
    chicken: Uuid,
    rice: Uuid,
    broccoli: Uuid,
    olive_oil: Uuid,
    eggs: Uuid,
    bread: Uuid,
    peanut_butter: Uuid,
    salmon: Uuid,
    sweet_potato: Uuid,
    apple: Uuid,
    yoghurt: Uuid,
    yoghurt_other_brand: Uuid,
    whey: Uuid,
    chocolate: Uuid,
    orange_juice: Uuid,
    protein_bar: Uuid,
}

async fn products(demo: &Demo) -> Result<Products> {
    Ok(Products {
        // Every nutrient the plausibility checks read, so the form shows them all passing.
        oats: create(
            demo,
            draft("Rolled oats", Unit::Gram, macros(372.0, 7.0, 13.5, 58.7))
                .nutrients(&[
                    ("energy-kj", 1557.0),
                    ("fiber", 10.0),
                    ("saturated-fat", 1.2),
                    ("sugars", 1.0),
                    ("salt", 0.01),
                ])
                .cost(0.25),
        )
        .await?,
        milk: create(
            demo,
            draft(
                "Semi-skimmed milk",
                Unit::Millilitre,
                macros(47.0, 1.6, 3.4, 4.8),
            )
            .nutrients(&[("saturated-fat", 1.0), ("sugars", 4.8), ("salt", 0.1)])
            .cost(0.11),
        )
        .await?,
        banana: create(
            demo,
            draft("Banana", Unit::Gram, macros(89.0, 0.3, 1.1, 22.8))
                .nutrients(&[("fiber", 2.6), ("sugars", 12.2)])
                .cost(0.22),
        )
        .await?,
        chicken: create(
            demo,
            draft("Chicken breast", Unit::Gram, macros(110.0, 1.2, 23.5, 0.0)).cost(1.10),
        )
        .await?,
        rice: create(
            demo,
            draft(
                "Basmati rice, dry",
                Unit::Gram,
                macros(350.0, 0.9, 8.0, 77.0),
            )
            .cost(0.30),
        )
        .await?,
        broccoli: create(
            demo,
            draft("Broccoli", Unit::Gram, macros(34.0, 0.4, 2.8, 4.4))
                .nutrients(&[("fiber", 2.6)])
                .cost(0.35),
        )
        .await?,
        olive_oil: create(
            demo,
            draft("Olive oil", Unit::Millilitre, macros(824.0, 91.6, 0.0, 0.0))
                .nutrients(&[("saturated-fat", 13.0)])
                .cost(0.90),
        )
        .await?,
        eggs: create(
            demo,
            draft("Eggs", Unit::Gram, macros(143.0, 9.5, 12.6, 0.7)).cost(0.55),
        )
        .await?,
        bread: create(
            demo,
            draft(
                "Wholemeal bread",
                Unit::Gram,
                macros(247.0, 3.4, 13.0, 41.0),
            )
            .nutrients(&[("fiber", 7.0), ("salt", 1.0)])
            .cost(0.40),
        )
        .await?,
        // No price: every meal and plan it appears in shows its cost as a floor.
        peanut_butter: create(
            demo,
            draft("Peanut butter", Unit::Gram, macros(588.0, 50.0, 25.0, 12.0)),
        )
        .await?,
        salmon: create(
            demo,
            draft("Salmon fillet", Unit::Gram, macros(208.0, 13.0, 20.0, 0.0)).cost(2.40),
        )
        .await?,
        sweet_potato: create(
            demo,
            draft("Sweet potato", Unit::Gram, macros(86.0, 0.1, 1.6, 20.0)).cost(0.28),
        )
        .await?,
        apple: create(
            demo,
            draft("Apple", Unit::Gram, macros(52.0, 0.2, 0.3, 13.8)).cost(0.25),
        )
        .await?,
        // Imported entries carry their code as provenance. The codes sit in GS1's in-store
        // range (prefix 20), which no real product is registered under.
        yoghurt: import(
            demo,
            "2000000000017",
            draft("Greek yoghurt 0%", Unit::Gram, macros(57.0, 0.2, 10.3, 3.6))
                .brand("Demo Dairy")
                .nutrients(&[("sugars", 3.6)])
                .cost(0.60),
        )
        .await?,
        // Same name, different brand: brand is identity, not decoration.
        yoghurt_other_brand: create(
            demo,
            draft("Greek yoghurt 0%", Unit::Gram, macros(54.0, 0.4, 9.0, 4.0))
                .brand("Hillside Farm")
                .cost(0.45),
        )
        .await?,
        whey: import(
            demo,
            "2000000000024",
            draft(
                "Whey protein, vanilla",
                Unit::Gram,
                macros(380.0, 6.0, 78.0, 6.0),
            )
            .brand("Demo Nutrition")
            .cost(3.20),
        )
        .await?,
        // Sugars above carbohydrates: opening its edit form shows a plausibility warning.
        chocolate: import(
            demo,
            "2000000000031",
            draft(
                "Dark chocolate 85%",
                Unit::Gram,
                macros(600.0, 46.0, 12.5, 19.0),
            )
            .brand("Demo Cocoa")
            .nutrients(&[("fiber", 12.0), ("sugars", 25.0), ("saturated-fat", 28.0)])
            .cost(1.80),
        )
        .await?,
        // kJ that does not match its kcal: the second kind of plausibility warning.
        orange_juice: import(
            demo,
            "2000000000048",
            draft(
                "Orange juice",
                Unit::Millilitre,
                macros(45.0, 0.2, 0.7, 10.4),
            )
            .brand("Demo Grove")
            .nutrients(&[("energy-kj", 450.0), ("sugars", 8.4)])
            .cost(0.19),
        )
        .await?,
        // Deleted at the end of the seed, after a meal and a plan name it.
        protein_bar: create(
            demo,
            draft("Protein bar", Unit::Gram, macros(360.0, 12.0, 33.0, 30.0)).cost(2.50),
        )
        .await?,
    })
}

async fn create(demo: &Demo, draft: ProductDraft) -> Result<Uuid> {
    Ok(demo.create_product(draft).await?.id)
}

async fn import(demo: &Demo, code: &str, draft: ProductDraft) -> Result<Uuid> {
    Ok(demo.import_product(code, draft).await?.id)
}

fn draft(name: &str, unit: Unit, macros: Macros) -> ProductDraft {
    ProductDraft {
        name: name.to_owned(),
        unit,
        macros,
        nutrients: Nutrients::new(),
        cost: None,
        brand: None,
    }
}

trait DraftExt {
    fn nutrients(self, nutrients: &[(&str, f64)]) -> Self;
    fn cost(self, cost: f64) -> Self;
    fn brand(self, brand: &str) -> Self;
}

impl DraftExt for ProductDraft {
    fn nutrients(mut self, nutrients: &[(&str, f64)]) -> Self {
        self.nutrients = nutrients
            .iter()
            .map(|(key, value)| ((*key).to_owned(), *value))
            .collect();
        self
    }

    fn cost(mut self, cost: f64) -> Self {
        self.cost = Some(cost);
        self
    }

    fn brand(mut self, brand: &str) -> Self {
        self.brand = Some(brand.to_owned());
        self
    }
}

fn macros(energy_kcal: f64, fat_g: f64, protein_g: f64, carbohydrates_g: f64) -> Macros {
    Macros {
        energy_kcal,
        fat_g,
        protein_g,
        carbohydrates_g,
    }
}

// categories

struct Categories {
    breakfast: Uuid,
    lunch: Uuid,
    dinner: Uuid,
    snack: Uuid,
    weekend_meal: Uuid,
    seasonal: Uuid,
    training: Uuid,
    rest: Uuid,
    weekend_plan: Uuid,
}

async fn categories(demo: &Demo) -> Result<Categories> {
    let category = async |name: &str, scope| {
        let draft = CategoryDraft {
            name: name.to_owned(),
            scope,
        };
        Ok::<_, Error>(demo.create_category(draft).await?.id)
    };

    // Created misspelt and renamed, the one change a category allows.
    let dinner = category("Diner", CategoryScope::Meal).await?;
    demo.rename_category(dinner, "Dinner").await?;

    Ok(Categories {
        breakfast: category("Breakfast", CategoryScope::Meal).await?,
        lunch: category("Lunch", CategoryScope::Meal).await?,
        dinner,
        snack: category("Snack", CategoryScope::Meal).await?,
        // One name in both scopes: each scope keeps its own list.
        weekend_meal: category("Weekend", CategoryScope::Meal).await?,
        seasonal: category("Seasonal", CategoryScope::Meal).await?,
        training: category("Training day", CategoryScope::DayPlan).await?,
        rest: category("Rest day", CategoryScope::DayPlan).await?,
        weekend_plan: category("Weekend", CategoryScope::DayPlan).await?,
    })
}

// meals

struct Meals {
    overnight_oats: Uuid,
    eggs_on_toast: Uuid,
    chicken_rice: Uuid,
    salmon_dinner: Uuid,
    yoghurt_whey: Uuid,
    bar_and_apple: Uuid,
    brunch: Uuid,
    soup: Uuid,
    leftovers: Uuid,
    old_smoothie: Uuid,
}

async fn meals(demo: &Demo, p: &Products, c: &Categories) -> Result<Meals> {
    let meal = async |label: &str, category_id: Option<Uuid>, servings: &[(Uuid, f64)]| {
        let draft = MealDraft {
            label: label.to_owned(),
            category_id,
            servings: servings
                .iter()
                .map(|&(product_id, amount)| Serving { product_id, amount })
                .collect(),
        };
        Ok::<_, Error>(demo.create_meal(draft).await?.entity.id)
    };

    Ok(Meals {
        // Peanut butter is unpriced, so this cost is incomplete.
        overnight_oats: meal(
            "Overnight oats",
            Some(c.breakfast),
            &[
                (p.oats, 60.0),
                (p.milk, 200.0),
                (p.banana, 100.0),
                (p.peanut_butter, 15.0),
            ],
        )
        .await?,
        eggs_on_toast: meal(
            "Scrambled eggs on toast",
            Some(c.breakfast),
            &[(p.eggs, 150.0), (p.bread, 80.0), (p.olive_oil, 5.0)],
        )
        .await?,
        chicken_rice: meal(
            "Chicken rice bowl",
            Some(c.lunch),
            &[
                (p.chicken, 150.0),
                (p.rice, 75.0),
                (p.broccoli, 150.0),
                (p.olive_oil, 10.0),
            ],
        )
        .await?,
        salmon_dinner: meal(
            "Salmon and sweet potato",
            Some(c.dinner),
            &[
                (p.salmon, 150.0),
                (p.sweet_potato, 250.0),
                (p.broccoli, 100.0),
                (p.olive_oil, 5.0),
            ],
        )
        .await?,
        yoghurt_whey: meal(
            "Yoghurt and whey",
            Some(c.snack),
            &[(p.yoghurt, 200.0), (p.whey, 30.0)],
        )
        .await?,
        // The bar is deleted later: this serving then renders as removed.
        bar_and_apple: meal(
            "Protein bar and apple",
            Some(c.snack),
            &[(p.protein_bar, 60.0), (p.apple, 150.0)],
        )
        .await?,
        brunch: meal(
            "Weekend brunch",
            Some(c.weekend_meal),
            &[
                (p.eggs, 100.0),
                (p.bread, 60.0),
                (p.salmon, 50.0),
                (p.yoghurt_other_brand, 150.0),
                (p.orange_juice, 250.0),
            ],
        )
        .await?,
        // Its category is deleted later, so it reads as being in a removed category.
        soup: meal(
            "Sweet potato soup",
            Some(c.seasonal),
            &[
                (p.sweet_potato, 300.0),
                (p.milk, 100.0),
                (p.olive_oil, 10.0),
            ],
        )
        .await?,
        leftovers: meal("Leftovers", None, &[(p.rice, 60.0), (p.chicken, 100.0)]).await?,
        // Deleted later: a deleted meal drops out of the plans that named it.
        old_smoothie: meal(
            "Old smoothie",
            Some(c.snack),
            &[(p.banana, 120.0), (p.milk, 250.0)],
        )
        .await?,
    })
}

// day plans

struct Plans {
    training_a: Uuid,
    training_b: Uuid,
    rest: Uuid,
    lazy_sunday: Uuid,
    travel: Uuid,
    old_cut: Uuid,
}

async fn day_plans(demo: &Demo, p: &Products, c: &Categories, m: &Meals) -> Result<Plans> {
    let plan = async |label: &str, category_id: Option<Uuid>, items: Vec<DayPlanItem>| {
        let draft = DayPlanDraft {
            label: label.to_owned(),
            category_id,
            items,
        };
        Ok::<_, Error>(demo.create_day_plan(draft).await?.id)
    };
    let meal = |meal_id| DayPlanItem::Meal { meal_id };
    let product = |product_id, amount| DayPlanItem::Product { product_id, amount };

    Ok(Plans {
        // Over the energy, protein and carbohydrate targets: the meters show the hatch.
        training_a: plan(
            "Training day A",
            Some(c.training),
            vec![
                meal(m.overnight_oats),
                meal(m.chicken_rice),
                meal(m.yoghurt_whey),
                product(p.banana, 120.0),
                meal(m.salmon_dinner),
                product(p.chocolate, 20.0),
            ],
        )
        .await?,
        training_b: plan(
            "Training day B",
            Some(c.training),
            vec![
                meal(m.eggs_on_toast),
                meal(m.leftovers),
                meal(m.bar_and_apple),
                meal(m.salmon_dinner),
                product(p.chocolate, 20.0),
            ],
        )
        .await?,
        // Every product priced and nothing removed: the one plan whose cost is complete.
        // The scoop of whey is what takes it to the protein target while leaving it under
        // the energy one, so the two are separate habits to the recovery comparison rather
        // than the same days under two names.
        rest: plan(
            "Rest day",
            Some(c.rest),
            vec![
                meal(m.eggs_on_toast),
                meal(m.chicken_rice),
                product(p.apple, 150.0),
                product(p.whey, 40.0),
                meal(m.salmon_dinner),
            ],
        )
        .await?,
        // Names a meal and a product that are deleted later: the meal drops out, the
        // product stays as a removed entry.
        lazy_sunday: plan(
            "Lazy Sunday",
            Some(c.weekend_plan),
            vec![
                meal(m.brunch),
                meal(m.old_smoothie),
                meal(m.soup),
                product(p.protein_bar, 60.0),
                product(p.chocolate, 30.0),
            ],
        )
        .await?,
        // Uncategorised and loose products only, with the unpriced peanut butter.
        travel: plan(
            "Travel day",
            None,
            vec![
                product(p.bread, 120.0),
                product(p.peanut_butter, 40.0),
                product(p.banana, 240.0),
                product(p.whey, 30.0),
                product(p.orange_juice, 330.0),
                product(p.apple, 150.0),
            ],
        )
        .await?,
        // Deleted later, while still on the calendar: that date reads as unplanned.
        old_cut: plan(
            "Old cutting plan",
            None,
            vec![meal(m.chicken_rice), meal(m.yoghurt_whey)],
        )
        .await?,
    })
}

// the shape of the history

/// How far back everything but the calendar reaches, in days.
///
/// Thirteen weeks, so that every window the insights read over is full rather than half
/// collected: three weeks for the energy balance, four for sleep regularity, eight for a
/// recovery baseline and thirteen for the behaviour comparisons.
const HISTORY_DAYS: i64 = 91;

/// Ten days away from home, in days back from the reference day: no scale and no strap, so
/// the weight graph, the sleep chart and both recovery series carry the same gap.
const AWAY_FROM: i64 = 43;
const AWAY_TO: i64 = 52;

fn away(back: i64) -> bool {
    (AWAY_FROM..=AWAY_TO).contains(&back)
}

/// The days the calendar plans a training day, which are the days the watch records a
/// workout: the two have to agree, or the recovery comparison is reading noise.
fn trains(date: Date) -> bool {
    matches!(
        date.weekday(),
        Weekday::Monday | Weekday::Wednesday | Weekday::Friday
    )
}

/// Rounded the way a figure of this kind is recorded: a tape and a scale to a tenth.
fn tenth(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

// calendar

/// Plans thirteen weeks back and three ahead.
///
/// Four weeks would fill the view, but the energy balance reads the last three weeks of
/// plans and the recovery comparison the last thirteen, so a shorter history would leave
/// both screens saying there is not enough yet.
const WEEKS_BACK: i64 = 13;
const WEEKS_AHEAD: i64 = 3;

async fn calendar(demo: &Demo, plans: &Plans, today: Date) -> Result<()> {
    let monday = today - Duration::days(i64::from(today.weekday().number_days_from_monday()));
    let first = monday - Duration::weeks(WEEKS_BACK);
    let last_week = WEEKS_BACK + WEEKS_AHEAD;

    for offset in 0..(last_week + 1) * 7 {
        let date = first + Duration::days(offset);
        let week = offset / 7;
        let plan = match date.weekday() {
            Weekday::Monday | Weekday::Friday => Some(plans.training_a),
            Weekday::Wednesday => Some(plans.training_b),
            Weekday::Tuesday | Weekday::Thursday => Some(plans.rest),
            // Every other Saturday is spent away.
            Weekday::Saturday => (week % 2 == 0).then_some(plans.travel),
            Weekday::Sunday => Some(plans.lazy_sunday),
        };
        // The last week is only half planned, as a week ahead usually is.
        if week == last_week && offset % 7 >= 3 {
            continue;
        }
        if let Some(plan) = plan {
            demo.assign_calendar_day(date, plan).await?;
        }
    }

    // Planned and then cleared, which is its own write.
    demo.unassign_calendar_day(monday + Duration::days(9))
        .await?;
    // Tomorrow keeps pointing at a plan that is deleted at the end of the seed.
    demo.assign_calendar_day(today + Duration::days(1), plans.old_cut)
        .await?;
    Ok(())
}

// weight

/// Thirteen weeks of weigh-ins ending today, losing about 0.4 kg a week under daily noise,
/// with the gaps a real log has: skipped days, and the ten days away without a scale.
async fn weight(demo: &Demo, today: Date) -> Result<()> {
    for back in (0..HISTORY_DAYS).rev() {
        let day = HISTORY_DAYS - 1 - back;
        let skipped = day % 5 == 3 || day % 11 == 7;
        if away(back) || (skipped && back != 0) {
            continue;
        }
        // Deterministic, so two seeds of the same day give the same chart.
        let noise = 0.45 * (day as f64 * 1.7).sin() + 0.25 * (day as f64 * 0.63).cos();
        let kilograms = 84.0 - 0.057 * day as f64 + noise;
        demo.set_weight_entry(today - Duration::days(back), tenth(kilograms))
            .await?;
    }
    Ok(())
}

// measurements

/// A waist every week and the other kinds now and then, over the same thirteen weeks.
///
/// The waist comes down about 1.5 cm in four weeks against 1.6 kg of trend, which is the
/// one pairing the app names. Height is measured once, because it is only there for the
/// ratio; the neck once and long ago, which is a series too short to compare; and the arm
/// never, so the picker has a kind with nothing behind it.
async fn measurements(demo: &Demo, today: Date) -> Result<()> {
    let record = async |back: i64, kind: MeasurementKind, value: f64| {
        demo.set_measurement(today - Duration::days(back), kind, tenth(value))
            .await?;
        Ok::<_, Error>(())
    };
    // Days from the start of the history, which every figure is a function of.
    let day = |back: i64| (HISTORY_DAYS - 1 - back) as f64;

    for back in (0..HISTORY_DAYS).step_by(7) {
        let day = day(back);
        record(
            back,
            MeasurementKind::Waist,
            91.4 - 0.055 * day + 0.35 * (day * 0.5).sin(),
        )
        .await?;
        if back % 14 == 0 {
            record(back, MeasurementKind::BodyFat, 21.8 - 0.035 * day).await?;
        }
        if back % 28 == 0 {
            record(back, MeasurementKind::Hips, 99.6 - 0.022 * day).await?;
            // The chest and the thigh hold while the waist comes down, which is why more
            // than one place is worth measuring.
            record(
                back,
                MeasurementKind::Chest,
                102.4 + 0.2 * (day * 0.4).sin(),
            )
            .await?;
            record(back, MeasurementKind::Thigh, 58.8 - 0.006 * day).await?;
        }
    }
    record(84, MeasurementKind::Height, 178.0).await?;
    record(56, MeasurementKind::Neck, 39.6).await?;
    Ok(())
}

// health

/// The strap arrived ten weeks in, in days back. Before that the nights came from a phone,
/// which knows when one started and ended and nothing in between.
const PHONE_ONLY_FROM: i64 = 70;

/// Two nights the strap was flat. The phone still counted the day's activity.
const FLAT_BATTERY: [i64; 2] = [17, 34];

/// Two days nothing counted the activity, which is not the same as a day of none: the
/// morning after each is left out of the workout comparison rather than counted as a rest.
const NO_ACTIVITY: [i64; 2] = [5, 26];

/// Thirteen weeks of nights, morning readings and daily activity, handed to `import_health`
/// as the raw records a sync arrives with, so the stored rows are what the rules summarised
/// rather than what this file thinks they should be.
///
/// The history is shaped the way a mostly-worn device records: a phone-only start, a strap
/// after it, two flat nights, the ten days away, three days ill three weeks back, and a hard
/// last week — which is why today's averages sit outside the normal range rather than in the
/// middle of it, where there would be nothing to read.
async fn health(demo: &Demo, today: Date) -> Result<()> {
    let first = today - Duration::days(HISTORY_DAYS - 1);
    let mut import = HealthImport::default();
    for back in (0..HISTORY_DAYS).rev() {
        Morning::new(today - Duration::days(back), back).record(&mut import);
    }
    // A morning sync on the reference day, so the Body screen reads as synced this morning
    // rather than at whatever hour the file happened to be built.
    demo.import_health(first, today, import, at(today, 7 * 60 + 10))
        .await?;
    Ok(())
}

/// One morning of the health history, and what was recording it.
///
/// Every figure is a function of `day`, so two seeds of the same reference day give the same
/// nights and the same charts.
struct Morning {
    date: Date,
    /// Days from the start of the history.
    day: f64,
    /// The strap was on overnight, which is what supplies the stages and the two markers.
    strap: bool,
    /// A night was recorded at all.
    slept: bool,
    /// A workout was recorded on this day.
    trained: bool,
    /// The day before this morning was a training day, which is what the night responds to.
    trained_before: bool,
    /// The day's activity was counted.
    activity: bool,
    /// How hard the last week and the three days ill push the markers.
    strain: f64,
    asleep_minutes: f64,
    /// Minutes from this morning's midnight, negative for the usual bedtime the evening
    /// before.
    bed_minute: f64,
    /// Two readings of each marker rather than one, the first stamped the evening before.
    split_readings: bool,
    /// A second app recorded this night too, shorter than the strap did.
    duplicated: bool,
    /// An afternoon nap on the day this morning opened.
    napped: bool,
}

impl Morning {
    fn new(date: Date, back: i64) -> Self {
        let day = (HISTORY_DAYS - 1 - back) as f64;
        let flat = FLAT_BATTERY.contains(&back);
        Self {
            date,
            day,
            strap: !away(back) && !flat && back < PHONE_ONLY_FROM,
            slept: !away(back) && !flat,
            trained: trains(date),
            trained_before: trains(date - Duration::days(1)),
            activity: !away(back) && !NO_ACTIVITY.contains(&back),
            strain: strain(back),
            asleep_minutes: 428.0 + 52.0 * (day * 1.31).sin() + 23.0 * (day * 0.47).cos(),
            // A different frequency from the duration's, so that going to bed early does not
            // also mean sleeping longer: the two habits are compared separately.
            bed_minute: -42.0 + 38.0 * (day * 0.83).sin() + 17.0 * (day * 2.17).cos(),
            split_readings: back % 17 == 3,
            duplicated: back == 12,
            napped: date.weekday() == Weekday::Sunday && back % 2 == 1,
        }
    }

    fn record(&self, import: &mut HealthImport) {
        if self.slept {
            import.sleep.push(self.night());
            // The same night from a second app, shorter: the longer session is the night,
            // and adding the two would sleep it twice.
            if self.duplicated {
                let night = self.night();
                import.sleep.push(SleepSession {
                    start: night.start + Duration::minutes(25),
                    end: night.end - Duration::minutes(15),
                    stages: Vec::new(),
                });
            }
            // A nap is its own session on the same morning, and is not part of the night.
            if self.napped {
                import.sleep.push(SleepSession {
                    start: at(self.date, 14 * 60),
                    end: at(self.date, 14 * 60 + 50),
                    stages: Vec::new(),
                });
            }
        }
        if self.strap {
            self.readings(import);
        }
        if self.activity {
            let workout = if self.trained { 430.0 } else { 0.0 };
            let active = 300.0 + 55.0 * (self.day * 0.55).sin() + workout;
            import.activity.push(ActivityDay {
                date: self.date,
                active_kcal: Some(active.round()),
                // A wearable's own total, which reads a couple of hundred above what the
                // weight trend implies, as a wrist estimate does.
                total_kcal: Some((1910.0 + active + 40.0 * (self.day * 0.33).cos()).round()),
                exercise_minutes: self
                    .trained
                    .then(|| (52.0 + 18.0 * (self.day * 0.9).sin().abs()).round()),
            });
        }
    }

    /// The night as the source recorded it: when it started, when it ended, and the stages in
    /// between when there was something to record them.
    fn night(&self) -> SleepSession {
        let start = at(self.date, self.bed_minute.round() as i32);
        let in_bed = self.asleep_minutes + self.awake_minutes();
        SleepSession {
            start,
            end: start + Duration::minutes(in_bed.round() as i64),
            stages: self.stages(start),
        }
    }

    /// Time awake in the night, which only the strap knows: a phone reports the bounds, and
    /// all of it counts as sleep.
    fn awake_minutes(&self) -> f64 {
        if self.strap {
            21.0 + 9.0 * (self.day * 1.9).sin()
        } else {
            0.0
        }
    }

    /// Light, deep, REM and one waking, laid end to end across the night.
    fn stages(&self, start: OffsetDateTime) -> Vec<SleepStage> {
        if !self.strap {
            return Vec::new();
        }
        let deep = self.asleep_minutes * (0.19 + 0.03 * (self.day * 0.7).sin());
        let rem = self.asleep_minutes * (0.21 + 0.04 * (self.day * 1.1).cos());
        let light = self.asleep_minutes - deep - rem;
        let spans = [
            (SleepStageKind::Light, light / 2.0),
            (SleepStageKind::Deep, deep),
            (SleepStageKind::Awake, self.awake_minutes()),
            (SleepStageKind::Rem, rem),
            (SleepStageKind::Light, light / 2.0),
        ];

        let mut at = start;
        spans
            .into_iter()
            .map(|(kind, minutes)| {
                let stage = SleepStage {
                    start: at,
                    end: at + Duration::seconds((minutes * 60.0).round() as i64),
                    kind,
                };
                at = stage.end;
                stage
            })
            .collect()
    }

    /// The three overnight readings.
    ///
    /// A night of seven hours reads better the next morning and the day after a workout
    /// reads worse, in both markers and in opposite directions, because that is the
    /// association the recovery screen is there to find. Bedtime is deliberately not one of
    /// them: a habit that turns out to make no clear difference is a state to show too.
    fn readings(&self, import: &mut HealthImport) {
        let rested = f64::from(self.asleep_minutes >= 420.0);
        let trained = f64::from(self.trained_before);
        let hrv = 63.0 + 5.4 * rested - 5.2 * trained - 7.0 * self.strain
            + 3.1 * (self.day * 1.13).sin()
            + 1.7 * (self.day * 0.61).cos();
        let rhr = 51.0 - 1.4 * rested
            + 1.3 * trained
            + 3.4 * self.strain
            + 1.1 * (self.day * 0.91).sin()
            + 0.7 * (self.day * 1.77).cos();
        let respiratory = 14.3 + 0.9 * self.strain + 0.4 * (self.day * 1.4).sin();

        for (readings, value, spread) in [
            (&mut import.heart_rate_variability, hrv, 2.5),
            (&mut import.resting_heart_rate, rhr, 0.8),
            (&mut import.respiratory_rate, respiratory, 0.3),
        ] {
            if self.split_readings {
                // One stamped at 23:40 and one at 05:10: both belong to this morning, and
                // the row keeps their mean rather than whichever synced last.
                readings.push(Reading {
                    at: at(self.date, -20),
                    value: value - spread,
                });
                readings.push(Reading {
                    at: at(self.date, 5 * 60 + 10),
                    value: value + spread,
                });
            } else {
                readings.push(Reading {
                    at: at(self.date, 4 * 60 + 40),
                    value,
                });
            }
        }
    }
}

/// How much load the markers are carrying on a given morning.
///
/// A hard last week, which is what the current verdict is read from, and three days ill
/// three weeks back, which is the spike in the middle of both charts.
fn strain(back: i64) -> f64 {
    if (1..=8).contains(&back) {
        1.0
    } else if (21..=23).contains(&back) {
        1.6
    } else {
        0.0
    }
}

/// A moment on the given date's own clock, in the one offset the demo lives in. Minutes may
/// be negative, which is the evening before: -20 is 23:40.
fn at(date: Date, minute: i32) -> OffsetDateTime {
    date.midnight().assume_offset(offset!(+2)) + Duration::minutes(i64::from(minute))
}

fn internal(reason: String) -> Error {
    Error::Internal { reason }
}
