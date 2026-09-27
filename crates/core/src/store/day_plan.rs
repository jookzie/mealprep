use uuid::Uuid;

use crate::{
    Result,
    domain::{DayPlan, DayPlanDraft},
};

/// Persistence of day plans and the order of their items.
///
/// Deletes are soft, and nothing deleted is ever returned.
#[trait_variant::make(Send)]
pub trait DayPlanStore {
    async fn create_day_plan(&self, draft: &DayPlanDraft) -> Result<DayPlan>;

    async fn get_day_plan(&self, id: Uuid) -> Result<DayPlan>;

    /// Lists every day plan, ordered by label.
    async fn list_day_plans(&self) -> Result<Vec<DayPlan>>;

    async fn update_day_plan(&self, id: Uuid, draft: &DayPlanDraft) -> Result<DayPlan>;

    async fn delete_day_plan(&self, id: Uuid) -> Result<()>;

    /// Reports whether a day plan other than the excluded one has the given label,
    /// ignoring case, within the given category; the uncategorised plans form one scope.
    async fn day_plan_label_taken(
        &self,
        label: &str,
        category_id: Option<Uuid>,
        exclude: Option<Uuid>,
    ) -> Result<bool>;
}
