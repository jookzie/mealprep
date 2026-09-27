use time::Date;

use crate::{Result, domain::WeightEntry};

/// Persistence of the weigh-ins.
///
/// Deletes are soft, and nothing deleted is ever returned.
#[trait_variant::make(Send)]
pub trait WeightStore {
    /// Records the weight for the given date, replacing whatever was there.
    async fn set_weight_entry(&self, date: Date, kilograms: f64) -> Result<WeightEntry>;

    /// Lists every weigh-in, in date order.
    ///
    /// The whole history rather than a range: the trend is smoothed from the first
    /// measurement onwards, so a window would give the days inside it the wrong trend.
    async fn list_weight_entries(&self) -> Result<Vec<WeightEntry>>;

    async fn delete_weight_entry(&self, date: Date) -> Result<()>;
}
