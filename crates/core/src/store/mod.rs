mod assistant;
mod calendar;
mod catalogue;
mod category;
mod day_plan;
mod meal;
mod product;
mod targets;
mod weight;

pub use self::{
    assistant::{Assistant, AssistantStore},
    calendar::CalendarStore,
    catalogue::Catalogue,
    category::CategoryStore,
    day_plan::DayPlanStore,
    meal::MealStore,
    product::ProductStore,
    targets::TargetsStore,
    weight::WeightStore,
};
