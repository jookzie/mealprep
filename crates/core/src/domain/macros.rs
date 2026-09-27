use std::{iter::Sum, ops::Add};

use serde::{Deserialize, Serialize};

/// The four figures the user plans against: per 100 units on a product, totals elsewhere.
///
/// Energy is kcal; the other three are grams.
#[derive(Copy, Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Macros {
    pub energy_kcal: f64,
    pub fat_g: f64,
    pub protein_g: f64,
    pub carbohydrates_g: f64,
}

impl Macros {
    pub const ZERO: Self = Self {
        energy_kcal: 0.0,
        fat_g: 0.0,
        protein_g: 0.0,
        carbohydrates_g: 0.0,
    };

    /// Scales these figures by the given factor, as a serving scales its product's.
    pub fn scale(self, factor: f64) -> Self {
        Self {
            energy_kcal: self.energy_kcal * factor,
            fat_g: self.fat_g * factor,
            protein_g: self.protein_g * factor,
            carbohydrates_g: self.carbohydrates_g * factor,
        }
    }

    /// Pairs each figure with its name, for messages that have to say which one is wrong.
    pub fn named(self) -> [(&'static str, f64); 4] {
        [
            ("energy", self.energy_kcal),
            ("fat", self.fat_g),
            ("protein", self.protein_g),
            ("carbohydrates", self.carbohydrates_g),
        ]
    }
}

/// One of the four figures, named as [`Macros`] sends it.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MacroField {
    EnergyKcal,
    FatG,
    ProteinG,
    CarbohydratesG,
}

impl Add for Macros {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            energy_kcal: self.energy_kcal + other.energy_kcal,
            fat_g: self.fat_g + other.fat_g,
            protein_g: self.protein_g + other.protein_g,
            carbohydrates_g: self.carbohydrates_g + other.carbohydrates_g,
        }
    }
}

impl Sum for Macros {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, Add::add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialises_in_camel_case() {
        let macros = Macros {
            energy_kcal: 1.0,
            fat_g: 2.0,
            protein_g: 3.0,
            carbohydrates_g: 4.0,
        };
        let json = serde_json::to_value(macros).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "energyKcal": 1.0, "fatG": 2.0, "proteinG": 3.0, "carbohydratesG": 4.0 })
        );
    }

    #[test]
    fn sums_and_scales() {
        let half = Macros {
            energy_kcal: 50.0,
            fat_g: 1.0,
            protein_g: 2.0,
            carbohydrates_g: 3.0,
        };
        let total: Macros = [half, half].into_iter().sum();
        assert_eq!(total, half.scale(2.0));
    }
}
