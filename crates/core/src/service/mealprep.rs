/// The business rules of Mealprep, over the given store, product catalogue and assistant.
///
/// Every operation the application offers is a method on this type. The files beside
/// this one hold those methods by the entity they act on, each block asking only for the
/// storage it uses, so a store for another database implements the same traits and
/// nothing here changes. The same holds for the other two: another catalogue, or another
/// model behind the assistant, is another crate implementing the trait.
#[derive(Clone, Debug)]
pub struct Mealprep<S, C, A> {
    pub(super) store: S,
    pub(super) catalogue: C,
    pub(super) assistant: A,
}

impl<S, C, A> Mealprep<S, C, A> {
    /// Creates the rules over the given store, catalogue and assistant.
    pub fn new(store: S, catalogue: C, assistant: A) -> Self {
        Self {
            store,
            catalogue,
            assistant,
        }
    }
}
