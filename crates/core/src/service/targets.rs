use super::{Mealprep, validate};
use crate::{
    Result,
    domain::{Macros, Targets},
    store::TargetsStore,
};

impl<S, C> Mealprep<S, C>
where
    S: TargetsStore,
{
    /// Fetches the daily targets, or `None` when the user has never set any.
    pub async fn get_targets(&self) -> Result<Option<Targets>> {
        self.store.get_targets().await
    }

    /// Sets the daily targets. They inform and never block a plan.
    pub async fn set_targets(&self, macros: Macros) -> Result<Targets> {
        let macros = validate::targets(macros)?;
        self.store.set_targets(macros).await
    }
}
