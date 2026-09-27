use uuid::Uuid;

use crate::{
    Result,
    domain::{Meal, MealDraft},
};

/// Persistence of meals and the order of their servings.
///
/// Deletes are soft, and nothing deleted is ever returned.
#[trait_variant::make(Send)]
pub trait MealStore {
    async fn create_meal(&self, draft: &MealDraft) -> Result<Meal>;

    async fn get_meal(&self, id: Uuid) -> Result<Meal>;

    /// Lists every meal, ordered by label.
    async fn list_meals(&self) -> Result<Vec<Meal>>;

    /// Lists those of the given meals that still exist, in no particular order.
    async fn list_meals_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Meal>>;

    async fn update_meal(&self, id: Uuid, draft: &MealDraft) -> Result<Meal>;

    async fn delete_meal(&self, id: Uuid) -> Result<()>;

    /// Reports whether a meal other than the excluded one has the given label, ignoring
    /// case, within the given category; the uncategorised meals form one scope.
    async fn meal_label_taken(
        &self,
        label: &str,
        category_id: Option<Uuid>,
        exclude: Option<Uuid>,
    ) -> Result<bool>;
}
