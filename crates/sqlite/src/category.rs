use mealprep_core::{
    Entity, Result,
    domain::{Category, CategoryDraft, CategoryScope},
    store::CategoryStore,
};
use uuid::Uuid;

use crate::{
    SqliteStore,
    row::{self, internal},
};

const INSERT: &str = "INSERT INTO categories (id, name, scope, created_at, updated_at) \
    VALUES (?, ?, ?, ?, ?) RETURNING *";
const SELECT: &str = "SELECT * FROM categories WHERE id = ? AND deleted_at IS NULL";
const LIST: &str = "SELECT * FROM categories \
    WHERE scope = ? AND deleted_at IS NULL ORDER BY name COLLATE NOCASE, id";
const RENAME: &str = "UPDATE categories SET name = ?, updated_at = ? \
    WHERE id = ? AND deleted_at IS NULL RETURNING *";
const DELETE: &str =
    "UPDATE categories SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL";
const NAME_TAKEN: &str = "SELECT count(*) FROM categories \
    WHERE name = ? COLLATE NOCASE AND scope = ? AND id <> ? AND deleted_at IS NULL";

impl CategoryStore for SqliteStore {
    async fn create_category(&self, draft: &CategoryDraft) -> Result<Category> {
        let now = row::now();
        let category: CategoryRow = sqlx::query_as(INSERT)
            .bind(Uuid::new_v4().to_string())
            .bind(&draft.name)
            .bind(draft.scope.as_str())
            .bind(&now)
            .bind(&now)
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;
        category.into_category()
    }

    async fn get_category(&self, id: Uuid) -> Result<Category> {
        let category: Option<CategoryRow> = sqlx::query_as(SELECT)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        let Some(category) = category else {
            return Err(row::not_found(Entity::Category, id));
        };
        category.into_category()
    }

    async fn list_categories(&self, scope: CategoryScope) -> Result<Vec<Category>> {
        let categories: Vec<CategoryRow> = sqlx::query_as(LIST)
            .bind(scope.as_str())
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        categories
            .into_iter()
            .map(CategoryRow::into_category)
            .collect()
    }

    async fn rename_category(&self, id: Uuid, name: &str) -> Result<Category> {
        let category: Option<CategoryRow> = sqlx::query_as(RENAME)
            .bind(name)
            .bind(row::now())
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        let Some(category) = category else {
            return Err(row::not_found(Entity::Category, id));
        };
        category.into_category()
    }

    async fn delete_category(&self, id: Uuid) -> Result<()> {
        let now = row::now();
        let deleted = sqlx::query(DELETE)
            .bind(&now)
            .bind(&now)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        if deleted.rows_affected() == 0 {
            return Err(row::not_found(Entity::Category, id));
        }
        Ok(())
    }

    async fn category_name_taken(
        &self,
        name: &str,
        scope: CategoryScope,
        exclude: Option<Uuid>,
    ) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(NAME_TAKEN)
            .bind(name)
            .bind(scope.as_str())
            .bind(row::scope_key(exclude))
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;
        Ok(count > 0)
    }
}

#[derive(sqlx::FromRow)]
struct CategoryRow {
    id: String,
    name: String,
    scope: String,
    created_at: String,
    updated_at: String,
}

impl CategoryRow {
    fn into_category(self) -> Result<Category> {
        let Self {
            id,
            name,
            scope,
            created_at,
            updated_at,
        } = self;

        let id = row::parse_id(&id)?;
        let scope = scope.parse().map_err(internal)?;
        let created_at = row::parse_timestamp(&created_at)?;
        let updated_at = row::parse_timestamp(&updated_at)?;

        Ok(Category {
            id,
            name,
            scope,
            created_at,
            updated_at,
        })
    }
}
