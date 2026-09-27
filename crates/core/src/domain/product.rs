use std::{collections::BTreeMap, fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{MacroField, Macros};
use crate::{Entity, Error};

/// Every nutrient beyond the four macros, per 100 units, keyed by name.
pub type Nutrients = BTreeMap<String, f64>;

/// A food the user plans with, imported from the catalogue or entered by hand.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub unit: Unit,
    pub macros: Macros,
    pub nutrients: Nutrients,
    /// What 100 units cost; absent rather than zero when the user never entered one,
    /// so that a sum can tell an unpriced product apart from a free one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    /// Who makes it. Two imports often share a name and differ only here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    /// The catalogue code the product was snapshotted from; absent when entered by hand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_code: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// What the user writes when creating or replacing a product.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductDraft {
    pub name: String,
    pub unit: Unit,
    pub macros: Macros,
    #[serde(default)]
    pub nutrients: Nutrients,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
}

/// A product as the external catalogue offers it, before import.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogueEntry {
    pub code: String,
    pub name: String,
    pub unit: Unit,
    pub macros: Macros,
    pub nutrients: Nutrients,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    /// The catalogue's own thumbnail. Images are licensed apart from the data, so they
    /// are linked and never stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// The macros the catalogue lacks, which read as zero in `macros` and have to be
    /// filled in from the package before the entry can be imported.
    #[serde(default)]
    pub missing_macros: Vec<MacroField>,
    pub source: EntrySource,
    /// The catalogue's own complaints about the figures, worded for people.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// When the entry was last edited, so a figure older than a recipe change can be told.
    #[serde(
        default,
        with = "time::serde::rfc3339::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub modified_at: Option<OffsetDateTime>,
}

/// Who supplied a catalogue entry, as far as the catalogue can tell.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntrySource {
    /// The brand owner, directly or through a product data feed such as GS1's.
    Manufacturer,
    /// Contributors transcribing the package.
    Community,
}

/// What a product's figures are measured per 100 of, and what its servings are in.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum Unit {
    #[serde(rename = "g")]
    Gram,
    #[serde(rename = "ml")]
    Millilitre,
}

impl Unit {
    /// Returns the symbol this unit is stored and sent as.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gram => "g",
            Self::Millilitre => "ml",
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Unit {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "g" => Ok(Self::Gram),
            "ml" => Ok(Self::Millilitre),
            _ => Err(Error::Invalid {
                entity: Entity::Product,
                reason: format!("unit {value:?} is neither \"g\" nor \"ml\""),
            }),
        }
    }
}
