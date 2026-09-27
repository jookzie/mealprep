use time::Date;

use crate::{
    Result,
    domain::{HealthDay, HealthSync},
};

/// Persistence of the imported health days.
///
/// Deletes are soft, and nothing deleted is ever returned.
#[trait_variant::make(Send)]
pub trait HealthStore {
    /// Replaces every day from `from` to `sync.through` inclusive with the given days, and
    /// records the sync.
    ///
    /// A day in the range that is not among `days` is deleted: the source no longer has
    /// anything for it. Days outside the range are left alone. All of it is one transaction,
    /// so a failed import leaves the previous one intact.
    async fn replace_health_days(
        &self,
        from: Date,
        days: &[HealthDay],
        sync: HealthSync,
    ) -> Result<()>;

    /// Lists every imported day, in date order.
    async fn list_health_days(&self) -> Result<Vec<HealthDay>>;

    /// The last sync, or `None` when nothing has been imported yet.
    async fn last_health_sync(&self) -> Result<Option<HealthSync>>;
}
