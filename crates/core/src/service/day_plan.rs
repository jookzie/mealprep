use uuid::Uuid;

use super::{
    Mealprep,
    derive::{self, MealIndex},
    validate,
};
use crate::{
    Entity, Error, Result,
    domain::{CategoryScope, DayPlan, DayPlanDraft, ResolvedDayPlan},
    store::{CategoryStore, DayPlanStore, MealStore, ProductStore},
};

impl<S, C> Mealprep<S, C>
where
    S: CategoryStore + DayPlanStore + MealStore + ProductStore,
{
    /// Lists every day plan with its items resolved and its totals derived.
    pub async fn list_day_plans(&self) -> Result<Vec<ResolvedDayPlan>> {
        let plans = self.store.list_day_plans().await?;
        self.resolve_day_plans(plans).await
    }

    pub async fn get_day_plan(&self, id: Uuid) -> Result<ResolvedDayPlan> {
        let plan = self.store.get_day_plan(id).await?;
        self.resolve_day_plan(plan).await
    }

    pub async fn create_day_plan(&self, draft: DayPlanDraft) -> Result<ResolvedDayPlan> {
        let draft = self.check_day_plan(draft, None).await?;
        let plan = self.store.create_day_plan(&draft).await?;
        self.resolve_day_plan(plan).await
    }

    /// Replaces the label, category and items of the day plan with the given id.
    pub async fn update_day_plan(&self, id: Uuid, draft: DayPlanDraft) -> Result<ResolvedDayPlan> {
        self.store.get_day_plan(id).await?;
        let draft = self.check_day_plan(draft, Some(id)).await?;
        let plan = self.store.update_day_plan(id, &draft).await?;
        self.resolve_day_plan(plan).await
    }

    /// Deletes the day plan with the given id. Dates it was assigned to keep the reference
    /// and read as unplanned.
    pub async fn delete_day_plan(&self, id: Uuid) -> Result<()> {
        self.store.delete_day_plan(id).await?;
        Ok(())
    }

    async fn resolve_day_plan(&self, plan: DayPlan) -> Result<ResolvedDayPlan> {
        let resolved = self.resolve_day_plans(vec![plan]).await?;
        resolved.into_iter().next().ok_or_else(|| Error::Internal {
            reason: "cannot resolve a single day plan".to_owned(),
        })
    }

    async fn resolve_day_plans(&self, plans: Vec<DayPlan>) -> Result<Vec<ResolvedDayPlan>> {
        let (meal_ids, loose_ids) = derive::item_ids(plans.iter().flat_map(|plan| &plan.items));
        let meals = self.store.list_meals_by_ids(&meal_ids).await?;

        let product_ids = {
            let mut ids = derive::product_ids(meals.iter().flat_map(|meal| &meal.servings));
            ids.extend(loose_ids);
            ids
        };
        let products = derive::index_products(self.store.list_products_by_ids(&product_ids).await?);
        let meals: MealIndex = meals
            .into_iter()
            .map(|meal| (meal.id, derive::derive_meal(meal, &products)))
            .collect();

        let resolved = plans
            .into_iter()
            .map(|plan| derive::resolve_day_plan(plan, &meals, &products))
            .collect();
        Ok(resolved)
    }

    async fn check_day_plan(
        &self,
        draft: DayPlanDraft,
        exclude: Option<Uuid>,
    ) -> Result<DayPlanDraft> {
        let draft = validate::day_plan(draft)?;
        if let Some(category_id) = draft.category_id {
            self.check_category(Entity::DayPlan, category_id, CategoryScope::DayPlan)
                .await?;
        }

        let (meal_ids, product_ids) = derive::item_ids(&draft.items);
        let meals = self.store.list_meals_by_ids(&meal_ids).await?;
        let missing = meal_ids
            .iter()
            .find(|id| meals.iter().all(|meal| meal.id != **id));
        if let Some(missing) = missing {
            let reason = format!("meal {missing} does not exist");
            return Err(Error::Invalid {
                entity: Entity::DayPlan,
                reason,
            });
        }
        self.check_products_exist(Entity::DayPlan, &product_ids)
            .await?;

        let taken = self
            .store
            .day_plan_label_taken(&draft.label, draft.category_id, exclude)
            .await?;
        if taken {
            return Err(Error::Conflict {
                entity: Entity::DayPlan,
                name: draft.label,
            });
        }
        Ok(draft)
    }
}
