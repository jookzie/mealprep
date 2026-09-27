use mealprep_core::domain::{CatalogueEntry, EntrySource, MacroField, Macros, Nutrients, Unit};
use serde::Deserialize;
use serde_json::{Map, Value};
use time::OffsetDateTime;

const PER_HUNDRED: &str = "_100g";
const ENERGY: &str = "energy-kcal";
const FAT: &str = "fat";
const PROTEIN: &str = "proteins";
const CARBOHYDRATES: &str = "carbohydrates";

/// Data sources that mean the brand owner supplied the entry: the producers' platform, and
/// the GS1 data pools that feed it.
const MANUFACTURER_SOURCES: &[&str] = &[
    "producers",
    "database-gdsn",
    "database-equadis",
    "database-codeonline",
    "database-agena3000",
];
/// Quality tags are in English whatever the product's language.
const TAG_LANGUAGE: &str = "en:";
/// Quality tags about the figures start with one of these; the rest are about ingredients,
/// packaging or the environmental score.
const FIGURE_TAGS: &[&str] = &["nutrition-", "energy-"];
/// Only per-100 figures are imported, so complaints about per-serving ones do not apply.
const SERVING: &str = "serving";

/// An entry as Open Food Facts returns it, with only the requested fields.
///
/// Every field may be missing or null in the wild, so each one is optional here.
#[derive(Deserialize)]
pub(crate) struct RawProduct {
    code: Option<String>,
    product_name: Option<String>,
    /// Preferred to `brands`, which is comma-separated with inconsistent casing.
    brands_tags: Option<Vec<String>>,
    image_front_small_url: Option<String>,
    nutrition_data_per: Option<String>,
    nutriments: Option<Map<String, Value>>,
    data_sources_tags: Option<Vec<String>>,
    data_quality_errors_tags: Option<Vec<String>>,
    data_quality_warnings_tags: Option<Vec<String>>,
    last_modified_t: Option<i64>,
}

impl RawProduct {
    /// Maps this entry onto the catalogue's own shape.
    ///
    /// Only numeric per-100 figures are kept, and the four macros move out of the nutrient
    /// set. The catalogue's quality errors come before its warnings.
    pub(crate) fn into_entry(self) -> CatalogueEntry {
        let Self {
            code,
            product_name,
            brands_tags,
            image_front_small_url,
            nutrition_data_per,
            nutriments,
            data_sources_tags,
            data_quality_errors_tags,
            data_quality_warnings_tags,
            last_modified_t,
        } = self;

        let mut nutrients: Nutrients = nutriments
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(key, value)| {
                let name = key.strip_suffix(PER_HUNDRED)?.to_owned();
                let value = value.as_f64()?;
                Some((name, value))
            })
            .collect();
        let mut missing_macros = Vec::new();
        let mut take = |name: &str, field: MacroField| {
            let value = nutrients.remove(name);
            if value.is_none() {
                missing_macros.push(field);
            }
            value.unwrap_or_default()
        };
        let macros = Macros {
            energy_kcal: take(ENERGY, MacroField::EnergyKcal),
            fat_g: take(FAT, MacroField::FatG),
            protein_g: take(PROTEIN, MacroField::ProteinG),
            carbohydrates_g: take(CARBOHYDRATES, MacroField::CarbohydratesG),
        };

        let unit = match nutrition_data_per.as_deref() {
            Some("100ml") => Unit::Millilitre,
            _ => Unit::Gram,
        };
        let brand = brands_tags.and_then(|brands| brands.into_iter().next());
        let source = if data_sources_tags
            .unwrap_or_default()
            .iter()
            .any(|tag| MANUFACTURER_SOURCES.contains(&tag.as_str()))
        {
            EntrySource::Manufacturer
        } else {
            EntrySource::Community
        };
        let warnings = data_quality_errors_tags
            .into_iter()
            .chain(data_quality_warnings_tags)
            .flatten()
            .filter_map(|tag| figure_warning(&tag))
            .collect();
        let modified_at =
            last_modified_t.and_then(|seconds| OffsetDateTime::from_unix_timestamp(seconds).ok());

        CatalogueEntry {
            code: code.unwrap_or_default(),
            name: product_name.unwrap_or_default().trim().to_owned(),
            unit,
            macros,
            nutrients,
            brand,
            image_url: image_front_small_url,
            missing_macros,
            source,
            warnings,
            modified_at,
        }
    }
}

/// Words a quality tag about the per-100 figures for people, so that
/// `en:nutrition-saturated-fat-greater-than-fat` reads "Nutrition saturated fat greater than
/// fat"; any other tag gives nothing.
fn figure_warning(tag: &str) -> Option<String> {
    let slug = tag.strip_prefix(TAG_LANGUAGE)?;
    if !FIGURE_TAGS.iter().any(|prefix| slug.starts_with(prefix)) || slug.contains(SERVING) {
        return None;
    }
    let words = slug.replace('-', " ");
    let mut chars = words.chars();
    let first = chars.next()?;
    Some(first.to_uppercase().chain(chars).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_numeric_per_hundred_figures_and_lifts_out_the_macros() {
        let raw: RawProduct = serde_json::from_value(serde_json::json!({
            "code": "3017620422003",
            "product_name": "Nutella ",
            "brands_tags": ["ferrero", "nutella"],
            "nutrition_data_per": "100g",
            "nutriments": {
                "energy-kcal_100g": 539,
                "fat_100g": 30.9,
                "proteins_100g": 6.3,
                "carbohydrates_100g": 57.5,
                "sugars_100g": 56.3,
                "sugars_unit": "g",
                "salt_serving": 0.1,
                "nova-group_100g": "4"
            }
        }))
        .unwrap();

        let entry = raw.into_entry();
        assert!(entry.missing_macros.is_empty());
        assert_eq!(entry.name, "Nutella");
        assert_eq!(entry.brand.as_deref(), Some("ferrero"));
        assert_eq!(entry.unit, Unit::Gram);
        assert_eq!(entry.macros.energy_kcal, 539.0);
        assert_eq!(entry.nutrients, [("sugars".to_owned(), 56.3)].into());
        assert_eq!(entry.source, EntrySource::Community);
        assert!(entry.warnings.is_empty());
        assert_eq!(entry.modified_at, None);
    }

    #[test]
    fn names_the_macros_an_entry_is_missing() {
        let raw: RawProduct = serde_json::from_value(serde_json::json!({
            "code": "1",
            "product_name": null,
            "brands_tags": null,
            "nutrition_data_per": "100ml",
            "nutriments": { "energy-kcal_100g": 42, "fat_100g": 0 }
        }))
        .unwrap();

        let entry = raw.into_entry();
        assert_eq!(
            entry.missing_macros,
            [MacroField::ProteinG, MacroField::CarbohydratesG]
        );
        assert_eq!(entry.macros.fat_g, 0.0);
        assert_eq!(entry.unit, Unit::Millilitre);
        assert_eq!(entry.brand, None);
    }

    #[test]
    fn reads_who_supplied_an_entry_and_what_the_catalogue_doubts_about_its_figures() {
        let raw: RawProduct = serde_json::from_value(serde_json::json!({
            "code": "3017620422003",
            "data_sources_tags": ["app-yuka", "apps", "producers", "producer-ferrero"],
            "data_quality_errors_tags": [
                "en:energy-value-in-kcal-does-not-match-value-in-kj",
                "en:nutrition-data-per-serving-missing-serving-size"
            ],
            "data_quality_warnings_tags": [
                "en:ingredients-en-ending-comma",
                "en:nutrition-saturated-fat-greater-than-fat"
            ],
            "last_modified_t": 1_789_382_259
        }))
        .unwrap();

        let entry = raw.into_entry();
        assert_eq!(entry.source, EntrySource::Manufacturer);
        assert_eq!(
            entry.warnings,
            [
                "Energy value in kcal does not match value in kj",
                "Nutrition saturated fat greater than fat"
            ]
        );
        assert_eq!(
            entry.modified_at,
            Some(OffsetDateTime::from_unix_timestamp(1_789_382_259).unwrap())
        );
    }

    #[test]
    fn a_community_source_database_does_not_make_an_entry_the_manufacturers() {
        let raw: RawProduct = serde_json::from_value(serde_json::json!({
            "data_sources_tags": ["database-foodrepo-openfood-ch", "databases", "app-yuka"]
        }))
        .unwrap();
        assert_eq!(raw.into_entry().source, EntrySource::Community);
    }
}
