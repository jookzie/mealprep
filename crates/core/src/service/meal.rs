use uuid::Uuid;

use super::{Mealprep, derive, validate};
use crate::{
    Entity, Error, Result,
    domain::{CategoryScope, Derived, Meal, MealDraft},
    store::{CategoryStore, MealStore, ProductStore},
};

impl<S, C> Mealprep<S, C>
where
    S: CategoryStore + MealStore + ProductStore,
{
    /// Lists every meal with its macros and cost derived.
    pub async fn list_meals(&self) -> Result<Vec<Derived<Meal>>> {
        let meals = self.store.list_meals().await?;
        let ids = derive::product_ids(meals.iter().flat_map(|meal| &meal.servings));
        let products = derive::index_products(self.store.list_products_by_ids(&ids).await?);
        let derived = meals
            .into_iter()
            .map(|meal| derive::derive_meal(meal, &products))
            .collect();
        Ok(derived)
    }

    pub async fn get_meal(&self, id: Uuid) -> Result<Derived<Meal>> {
        let meal = self.store.get_meal(id).await?;
        self.derive_meal(meal).await
    }

    pub async fn create_meal(&self, draft: MealDraft) -> Result<Derived<Meal>> {
        let draft = self.check_meal(draft, None).await?;
        let meal = self.store.create_meal(&draft).await?;
        self.derive_meal(meal).await
    }

    /// Replaces the label, category and servings of the meal with the given id.
    pub async fn update_meal(&self, id: Uuid, draft: MealDraft) -> Result<Derived<Meal>> {
        self.store.get_meal(id).await?;
        let draft = self.check_meal(draft, Some(id)).await?;
        let meal = self.store.update_meal(id, &draft).await?;
        self.derive_meal(meal).await
    }

    /// Deletes the meal with the given id. Day plans that hold it drop it from their totals.
    pub async fn delete_meal(&self, id: Uuid) -> Result<()> {
        self.store.delete_meal(id).await?;
        Ok(())
    }

    async fn derive_meal(&self, meal: Meal) -> Result<Derived<Meal>> {
        let ids = derive::product_ids(&meal.servings);
        let products = derive::index_products(self.store.list_products_by_ids(&ids).await?);
        Ok(derive::derive_meal(meal, &products))
    }

    async fn check_meal(&self, draft: MealDraft, exclude: Option<Uuid>) -> Result<MealDraft> {
        let draft = validate::meal(draft)?;
        if let Some(category_id) = draft.category_id {
            self.check_category(Entity::Meal, category_id, CategoryScope::Meal)
                .await?;
        }

        let product_ids = derive::product_ids(&draft.servings);
        self.check_products_exist(Entity::Meal, &product_ids)
            .await?;

        let taken = self
            .store
            .meal_label_taken(&draft.label, draft.category_id, exclude)
            .await?;
        if taken {
            return Err(Error::Conflict {
                entity: Entity::Meal,
                name: draft.label,
            });
        }
        Ok(draft)
    }
}
