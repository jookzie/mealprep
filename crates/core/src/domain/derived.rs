use serde::{Deserialize, Serialize};

use super::{Cost, Macros};

/// A composition together with the figures derived from what it holds.
///
/// It serialises as the entity's own fields with `macros` and `cost` beside them.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Derived<T> {
    #[serde(flatten)]
    pub entity: T,
    pub macros: Macros,
    pub cost: Cost,
}
