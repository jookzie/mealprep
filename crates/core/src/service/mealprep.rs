/// The business rules of Mealprep, over the given store and product catalogue.
///
/// Every operation the application offers is a method on this type. The files beside
/// this one hold those methods by the entity they act on, each block asking only for the
/// storage it uses, so a store for another database implements the same traits and
/// nothing here changes.
#[derive(Clone, Debug)]
pub struct Mealprep<S, C> {
    pub(super) store: S,
    pub(super) catalogue: C,
}

impl<S, C> Mealprep<S, C> {
    /// Creates the rules over the given store and catalogue.
    pub fn new(store: S, catalogue: C) -> Self {
        Self { store, catalogue }
    }
}
