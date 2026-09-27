use std::collections::HashMap;

use mealprep_core::{
    Entity, Result,
    domain::{Meal, MealDraft, Serving},
    store::MealStore,
};
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::{
    SqliteStore,
    row::{self, internal},
};

const INSERT: &str = "INSERT INTO meals (id, label, category_id, created_at, updated_at) \
    VALUES (?, ?, ?, ?, ?) RETURNING *";
const SELECT: &str = "SELECT * FROM meals WHERE id = ? AND deleted_at IS NULL";
const LIST: &str = "SELECT * FROM meals WHERE deleted_at IS NULL ORDER BY label COLLATE NOCASE, id";
const LIST_BY_IDS: &str = "SELECT * FROM meals \
    WHERE id IN (SELECT value FROM json_each(?)) AND deleted_at IS NULL";
const UPDATE: &str = "UPDATE meals SET label = ?, category_id = ?, updated_at = ? \
    WHERE id = ? AND deleted_at IS NULL RETURNING *";
const DELETE: &str =
    "UPDATE meals SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL";
// coalesce, not a comparison: NULL = NULL is never true, so without the fold every
// uncategorised meal would be unique by definition.
const LABEL_TAKEN: &str = "SELECT count(*) FROM meals \
    WHERE label = ? COLLATE NOCASE AND coalesce(category_id, '') = ? AND id <> ? AND deleted_at IS NULL";

const INSERT_SERVING: &str = "INSERT INTO meal_servings \
    (id, meal_id, product_id, amount, position, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)";
const LIST_SERVINGS: &str = "SELECT meal_id, product_id, amount FROM meal_servings \
    WHERE meal_id IN (SELECT value FROM json_each(?)) AND deleted_at IS NULL \
    ORDER BY meal_id, position";
const DELETE_SERVINGS: &str = "UPDATE meal_servings SET deleted_at = ?, updated_at = ? \
    WHERE meal_id = ? AND deleted_at IS NULL";

impl MealStore for SqliteStore {
    async fn create_meal(&self, draft: &MealDraft) -> Result<Meal> {
        let id = Uuid::new_v4().to_string();
        let now = row::now();
        let mut transaction = self.pool.begin().await.map_err(internal)?;

        let meal: MealRow = sqlx::query_as(INSERT)
            .bind(&id)
            .bind(&draft.label)
            .bind(draft.category_id.map(|category_id| category_id.to_string()))
            .bind(&now)
            .bind(&now)
            .fetch_one(&mut *transaction)
            .await
            .map_err(internal)?;
        insert_servings(&mut transaction, &id, &draft.servings, &now).await?;

        transaction.commit().await.map_err(internal)?;
        meal.into_meal(draft.servings.clone())
    }

    async fn get_meal(&self, id: Uuid) -> Result<Meal> {
        let meal: Option<MealRow> = sqlx::query_as(SELECT)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        let Some(meal) = meal else {
            return Err(row::not_found(Entity::Meal, id));
        };
        let meals = attach_servings(&self.pool, vec![meal]).await?;
        meals
            .into_iter()
            .next()
            .ok_or_else(|| row::not_found(Entity::Meal, id))
    }

    async fn list_meals(&self) -> Result<Vec<Meal>> {
        let meals: Vec<MealRow> = sqlx::query_as(LIST)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        attach_servings(&self.pool, meals).await
    }

    async fn list_meals_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Meal>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let meals: Vec<MealRow> = sqlx::query_as(LIST_BY_IDS)
            .bind(row::id_list(ids))
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        attach_servings(&self.pool, meals).await
    }

    async fn update_meal(&self, id: Uuid, draft: &MealDraft) -> Result<Meal> {
        let now = row::now();
        let meal_id = id.to_string();
        let mut transaction = self.pool.begin().await.map_err(internal)?;

        let meal: Option<MealRow> = sqlx::query_as(UPDATE)
            .bind(&draft.label)
            .bind(draft.category_id.map(|category_id| category_id.to_string()))
            .bind(&now)
            .bind(&meal_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(internal)?;
        let Some(meal) = meal else {
            return Err(row::not_found(Entity::Meal, id));
        };
        sqlx::query(DELETE_SERVINGS)
            .bind(&now)
            .bind(&now)
            .bind(&meal_id)
            .execute(&mut *transaction)
            .await
            .map_err(internal)?;
        insert_servings(&mut transaction, &meal_id, &draft.servings, &now).await?;

        transaction.commit().await.map_err(internal)?;
        meal.into_meal(draft.servings.clone())
    }

    async fn delete_meal(&self, id: Uuid) -> Result<()> {
        let now = row::now();
        let deleted = sqlx::query(DELETE)
            .bind(&now)
            .bind(&now)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        if deleted.rows_affected() == 0 {
            return Err(row::not_found(Entity::Meal, id));
        }
        Ok(())
    }

    async fn meal_label_taken(
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
struct MealRow {
    id: String,
    label: String,
    category_id: Option<String>,
    created_at: String,
    updated_at: String,
}

impl MealRow {
    fn into_meal(self, servings: Vec<Serving>) -> Result<Meal> {
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

        Ok(Meal {
            id,
            label,
            category_id,
            servings,
            created_at,
            updated_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ServingRow {
    meal_id: String,
    product_id: String,
    amount: f64,
}

/// Fills in the servings of the given meals, in their stored order.
async fn attach_servings(pool: &SqlitePool, meals: Vec<MealRow>) -> Result<Vec<Meal>> {
    let ids = meals
        .iter()
        .map(|meal| row::parse_id(&meal.id))
        .collect::<Result<Vec<_>>>()?;
    let servings: Vec<ServingRow> = sqlx::query_as(LIST_SERVINGS)
        .bind(row::id_list(&ids))
        .fetch_all(pool)
        .await
        .map_err(internal)?;

    let by_meal = {
        let mut by_meal: HashMap<String, Vec<Serving>> = HashMap::new();
        for serving in servings {
            let ServingRow {
                meal_id,
                product_id,
                amount,
            } = serving;
            let product_id = row::parse_id(&product_id)?;
            by_meal
                .entry(meal_id)
                .or_default()
                .push(Serving { product_id, amount });
        }
        by_meal
    };

    meals
        .into_iter()
        .map(|meal| {
            let servings = by_meal.get(&meal.id).cloned().unwrap_or_default();
            meal.into_meal(servings)
        })
        .collect()
}

async fn insert_servings(
    transaction: &mut Transaction<'_, Sqlite>,
    meal_id: &str,
    servings: &[Serving],
    now: &str,
) -> Result<()> {
    for (position, serving) in servings.iter().enumerate() {
        let position = i64::try_from(position).map_err(internal)?;
        sqlx::query(INSERT_SERVING)
            .bind(Uuid::new_v4().to_string())
            .bind(meal_id)
            .bind(serving.product_id.to_string())
            .bind(serving.amount)
            .bind(position)
            .bind(now)
            .bind(now)
            .execute(&mut **transaction)
            .await
            .map_err(internal)?;
    }
    Ok(())
}
