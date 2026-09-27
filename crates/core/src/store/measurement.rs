use time::Date;

use crate::{
    Result,
    domain::{Measurement, MeasurementKind},
};

/// Persistence of the body measurements.
///
/// Deletes are soft, and nothing deleted is ever returned.
#[trait_variant::make(Send)]
pub trait MeasurementStore {
    /// Records the value of the given kind for the given date, replacing whatever was there.
    async fn set_measurement(
        &self,
        date: Date,
        kind: MeasurementKind,
        value: f64,
    ) -> Result<Measurement>;

    /// Lists every measurement of every kind, in date order.
    async fn list_measurements(&self) -> Result<Vec<Measurement>>;

    async fn delete_measurement(&self, date: Date, kind: MeasurementKind) -> Result<()>;
}
