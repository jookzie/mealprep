use std::collections::HashMap;

use mealprep_core::{
    Entity, Result,
    domain::{DayPlan, DayPlanDraft, DayPlanItem},
    store::DayPlanStore,
};
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::{
    SqliteStore,
    row::{self, internal},
};

const INSERT: &str = "INSERT INTO day_plans (id, label, category_id, created_at, updated_at) \
    VALUES (?, ?, ?, ?, ?) RETURNING *";
const SELECT: &str = "SELECT * FROM day_plans WHERE id = ? AND deleted_at IS NULL";
const LIST: &str =
    "SELECT * FROM day_plans WHERE deleted_at IS NULL ORDER BY label COLLATE NOCASE, id";
const UPDATE: &str = "UPDATE day_plans SET label = ?, category_id = ?, updated_at = ? \
    WHERE id = ? AND deleted_at IS NULL RETURNING *";
const DELETE: &str =
    "UPDATE day_plans SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL";
// coalesce, not a comparison: NULL = NULL is never true, so without the fold every
// uncategorised plan would be unique by definition.
const LABEL_TAKEN: &str = "SELECT count(*) FROM day_plans \
    WHERE label = ? COLLATE NOCASE AND coalesce(category_id, '') = ? AND id <> ? AND deleted_at IS NULL";

const INSERT_ITEM: &str = "INSERT INTO day_plan_items \
    (id, day_plan_id, kind, meal_id, product_id, amount, position, created_at, updated_at) \
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
const LIST_ITEMS: &str = "SELECT day_plan_id, kind, meal_id, product_id, amount FROM day_plan_items \
    WHERE day_plan_id IN (SELECT value FROM json_each(?)) AND deleted_at IS NULL \
    ORDER BY day_plan_id, position";
const DELETE_ITEMS: &str = "UPDATE day_plan_items SET deleted_at = ?, updated_at = ? \
    WHERE day_plan_id = ? AND deleted_at IS NULL";

impl DayPlanStore for SqliteStore {
    async fn create_day_plan(&self, draft: &DayPlanDraft) -> Result<DayPlan> {
        let id = Uuid::new_v4().to_string();
        let now = row::now();
        let mut transaction = self.pool.begin().await.map_err(internal)?;

        let plan: DayPlanRow = sqlx::query_as(INSERT)
            .bind(&id)
            .bind(&draft.label)
            .bind(draft.category_id.map(|category_id| category_id.to_string()))
            .bind(&now)
            .bind(&now)
            .fetch_one(&mut *transaction)
            .await
            .map_err(internal)?;
        insert_items(&mut transaction, &id, &draft.items, &now).await?;

        transaction.commit().await.map_err(internal)?;
        plan.into_day_plan(draft.items.clone())
    }

    async fn get_day_plan(&self, id: Uuid) -> Result<DayPlan> {
        let plan: Option<DayPlanRow> = sqlx::query_as(SELECT)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        let Some(plan) = plan else {
            return Err(row::not_found(Entity::DayPlan, id));
        };
        let plans = attach_items(&self.pool, vec![plan]).await?;
        plans
            .into_iter()
            .next()
            .ok_or_else(|| row::not_found(Entity::DayPlan, id))
    }

    async fn list_day_plans(&self) -> Result<Vec<DayPlan>> {
        let plans: Vec<DayPlanRow> = sqlx::query_as(LIST)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        attach_items(&self.pool, plans).await
    }

    async fn update_day_plan(&self, id: Uuid, draft: &DayPlanDraft) -> Result<DayPlan> {
        let now = row::now();
        let day_plan_id = id.to_string();
        let mut transaction = self.pool.begin().await.map_err(internal)?;

        let plan: Option<DayPlanRow> = sqlx::query_as(UPDATE)
            .bind(&draft.label)
            .bind(draft.category_id.map(|category_id| category_id.to_string()))
            .bind(&now)
            .bind(&day_plan_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(internal)?;
        let Some(plan) = plan else {
            return Err(row::not_found(Entity::DayPlan, id));
        };
        sqlx::query(DELETE_ITEMS)
            .bind(&now)
            .bind(&now)
            .bind(&day_plan_id)
            .execute(&mut *transaction)
            .await
            .map_err(internal)?;
        insert_items(&mut transaction, &day_plan_id, &draft.items, &now).await?;

        transaction.commit().await.map_err(internal)?;
        plan.into_day_plan(draft.items.clone())
    }

    async fn delete_day_plan(&self, id: Uuid) -> Result<()> {
        let now = row::now();
        let deleted = sqlx::query(DELETE)
            .bind(&now)
            .bind(&now)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        if deleted.rows_affected() == 0 {
            return Err(row::not_found(Entity::DayPlan, id));
        }
        Ok(())
    }

    async fn day_plan_label_taken(
        &self,
        label: &str,
        category_id: Option<Uuid>,
        exclude: Option<Uuid>,
    ) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(LABEL_TAKEN)
            .bind(label)
            .bind(row::scope_key(category_id))
            .bind(row::scope_key(exclude))
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;
        Ok(count > 0)
    }
}

#[derive(sqlx::FromRow)]
struct DayPlanRow {
    id: String,
    label: String,
    category_id: Option<String>,
    created_at: String,
    updated_at: String,
}

impl DayPlanRow {
    fn into_day_plan(self, items: Vec<DayPlanItem>) -> Result<DayPlan> {
        let Self {
            id,
            label,
            category_id,
            created_at,
            updated_at,
        } = self;

        let id = row::parse_id(&id)?;
        let category_id = row::parse_optional_id(category_id.as_deref())?;
        let created_at = row::parse_timestamp(&created_at)?;
        let updated_at = row::parse_timestamp(&updated_at)?;

        Ok(DayPlan {
            id,
            label,
            category_id,
            items,
            created_at,
            updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ItemRow {
    day_plan_id: String,
    kind: String,
    meal_id: Option<String>,
    product_id: Option<String>,
    amount: Option<f64>,
}

impl ItemRow {
    fn into_item(self) -> Result<DayPlanItem> {
        let Self {
            day_plan_id,
            kind,
            meal_id,
            product_id,
            amount,
        } = self;

        match (kind.as_str(), meal_id, product_id, amount) {
            ("meal", Some(meal_id), None, None) => {
                let meal_id = row::parse_id(&meal_id)?;
                Ok(DayPlanItem::Meal { meal_id })
            }
            ("product", None, Some(product_id), Some(amount)) => {
                let product_id = row::parse_id(&product_id)?;
                Ok(DayPlanItem::Product { product_id, amount })
            }
            _ => Err(internal(format!(
                "cannot read an item of kind {kind:?} in day plan {day_plan_id}"
            ))),
        }
    }
}

/// Fills in the items of the given day plans, in their stored order.
async fn attach_items(pool: &SqlitePool, plans: Vec<DayPlanRow>) -> Result<Vec<DayPlan>> {
    let ids = plans
        .iter()
        .map(|plan| row::parse_id(&plan.id))
        .collect::<Result<Vec<_>>>()?;
    let items: Vec<ItemRow> = sqlx::query_as(LIST_ITEMS)
        .bind(row::id_list(&ids))
        .fetch_all(pool)
        .await
        .map_err(internal)?;

    let by_plan = {
        let mut by_plan: HashMap<String, Vec<DayPlanItem>> = HashMap::new();
        for item in items {
            let day_plan_id = item.day_plan_id.clone();
            let item = item.into_item()?;
            by_plan.entry(day_plan_id).or_default().push(item);
        }
        by_plan
    };

    plans
        .into_iter()
        .map(|plan| {
            let items = by_plan.get(&plan.id).cloned().unwrap_or_default();
            plan.into_day_plan(items)
        })
        .collect()
}

async fn insert_items(
    transaction: &mut Transaction<'_, Sqlite>,
    day_plan_id: &str,
    items: &[DayPlanItem],
    now: &str,
) -> Result<()> {
    for (position, item) in items.iter().enumerate() {
        let position = i64::try_from(position).map_err(internal)?;
        let (kind, meal_id, product_id, amount) = match item {
            DayPlanItem::Meal { meal_id } => ("meal", Some(meal_id.to_string()), None, None),
            DayPlanItem::Product { product_id, amount } => {
                ("product", None, Some(product_id.to_string()), Some(*amount))
            }
        };
        sqlx::query(INSERT_ITEM)
            .bind(Uuid::new_v4().to_string())
            .bind(day_plan_id)
            .bind(kind)
            .bind(meal_id)
            .bind(product_id)
            .bind(amount)
            .bind(position)
            .bind(now)
            .bind(now)
            .execute(&mut **transaction)
            .await
            .map_err(internal)?;
    }
    Ok(())
}
