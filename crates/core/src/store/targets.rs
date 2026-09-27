use crate::{
    Result,
    domain::{Macros, Targets},
};

/// Persistence of the single set of daily targets.
#[trait_variant::make(Send)]
pub trait TargetsStore {
    /// Fetches the targets, or `None` when the user has never set any.
    async fn get_targets(&self) -> Result<Option<Targets>>;

    async fn set_targets(&self, macros: Macros) -> Result<Targets>;
}
