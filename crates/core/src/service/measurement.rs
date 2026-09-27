use time::Date;

use super::{Mealprep, validate};
use crate::{
    Result,
    domain::{Measurement, MeasurementKind},
    store::MeasurementStore,
};

impl<S, C> Mealprep<S, C>
where
    S: MeasurementStore,
{
    /// Lists every measurement of every kind, in date order.
    pub async fn list_measurements(&self) -> Result<Vec<Measurement>> {
        self.store.list_measurements().await
    }

    /// Records the value of the given kind for the given date, replacing whatever was there.
    pub async fn set_measurement(
        &self,
        date: Date,
        kind: MeasurementKind,
        value: f64,
    ) -> Result<Measurement> {
        let value = validate::measurement(kind, value)?;
        self.store.set_measurement(date, kind, value).await
    }

    pub async fn delete_measurement(&self, date: Date, kind: MeasurementKind) -> Result<()> {
        self.store.delete_measurement(date, kind).await
    }
}
