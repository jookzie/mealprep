use uuid::Uuid;

use crate::{
    Result,
    domain::{DayPlan, DayPlanDraft},
};

/// Persistence of day plans and the order of their items.
///
/// Deletes are soft, and nothing deleted is ever returned.
///
/// A day plan carrying a `source_plan_id` is one date's own copy of another plan, made by
/// [`Self::create_day_plan_copy`]. A copy is not a plan the user keeps, so it is excluded
/// from the plan list and from label uniqueness, and it is reached only by its id — which
/// is how nothing but the calendar day it was made for can ever point at it.
#[trait_variant::make(Send)]
pub trait DayPlanStore {
    async fn create_day_plan(&self, draft: &DayPlanDraft) -> Result<DayPlan>;

    /// Stores the given draft as one date's own copy of the plan with the given id.
    ///
    /// The copy carries `source_plan_id`, keeps the label and category it was copied
    /// from, and is left out of [`Self::list_day_plans`] and of
    /// [`Self::day_plan_label_taken`]. Copying a copy records the original as the source,
    /// so the chain never grows.
    async fn create_day_plan_copy(
        &self,
        source_plan_id: Uuid,
        draft: &DayPlanDraft,
    ) -> Result<DayPlan>;

    /// Fetches the plan with the given id, a day-local copy included.
    async fn get_day_plan(&self, id: Uuid) -> Result<DayPlan>;

    /// Lists every day plan the user keeps, ordered by label. Day-local copies are
    /// excluded.
    async fn list_day_plans(&self) -> Result<Vec<DayPlan>>;

    /// Lists those of the given plans that still exist, day-local copies included, in no
    /// particular order.
    async fn list_day_plans_by_ids(&self, ids: &[Uuid]) -> Result<Vec<DayPlan>>;

    async fn update_day_plan(&self, id: Uuid, draft: &DayPlanDraft) -> Result<DayPlan>;

    async fn delete_day_plan(&self, id: Uuid) -> Result<()>;

    /// Reports whether a day plan other than the excluded one has the given label,
    /// ignoring case, within the given category; the uncategorised plans form one scope.
    /// Day-local copies do not count: they share the label they were copied from.
    async fn day_plan_label_taken(
        &self,
        label: &str,
        category_id: Option<Uuid>,
        exclude: Option<Uuid>,
    ) -> Result<bool>;
}
