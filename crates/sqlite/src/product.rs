use mealprep_core::{
    Entity, Result,
    domain::{Macros, Product, ProductDraft},
    store::ProductStore,
};
use uuid::Uuid;

use crate::{
    SqliteStore,
    row::{self, internal},
};

const INSERT: &str = "INSERT INTO products \
    (id, name, unit, energy_kcal, fat_g, protein_g, carbohydrates_g, nutrients, cost, brand, source_code, created_at, updated_at) \
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING *";
const SELECT: &str = "SELECT * FROM products WHERE id = ? AND deleted_at IS NULL";
const LIST: &str =
    "SELECT * FROM products WHERE deleted_at IS NULL ORDER BY name COLLATE NOCASE, id";
const LIST_BY_IDS: &str = "SELECT * FROM products \
    WHERE id IN (SELECT value FROM json_each(?)) AND deleted_at IS NULL";
const UPDATE: &str = "UPDATE products \
    SET name = ?, unit = ?, energy_kcal = ?, fat_g = ?, protein_g = ?, carbohydrates_g = ?, nutrients = ?, cost = ?, brand = ?, updated_at = ? \
    WHERE id = ? AND deleted_at IS NULL RETURNING *";
const DELETE: &str =
    "UPDATE products SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL";

impl ProductStore for SqliteStore {
    async fn create_product(
        &self,
        draft: &ProductDraft,
        source_code: Option<&str>,
    ) -> Result<Product> {
        let id = Uuid::new_v4().to_string();
        let now = row::now();
        let nutrients = serde_json::to_string(&draft.nutrients).map_err(internal)?;
        let product: ProductRow = sqlx::query_as(INSERT)
            .bind(id)
            .bind(&draft.name)
            .bind(draft.unit.as_str())
            .bind(draft.macros.energy_kcal)
            .bind(draft.macros.fat_g)
            .bind(draft.macros.protein_g)
            .bind(draft.macros.carbohydrates_g)
            .bind(nutrients)
            .bind(draft.cost)
            .bind(draft.brand.as_deref())
            .bind(source_code)
            .bind(&now)
            .bind(&now)
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;
        product.into_product()
    }

    async fn get_product(&self, id: Uuid) -> Result<Product> {
        let product: Option<ProductRow> = sqlx::query_as(SELECT)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        let Some(product) = product else {
            return Err(row::not_found(Entity::Product, id));
        };
        product.into_product()
    }

    async fn list_products(&self) -> Result<Vec<Product>> {
        let products: Vec<ProductRow> = sqlx::query_as(LIST)
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        products.into_iter().map(ProductRow::into_product).collect()
    }

    async fn list_products_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Product>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let products: Vec<ProductRow> = sqlx::query_as(LIST_BY_IDS)
            .bind(row::id_list(ids))
            .fetch_all(&self.pool)
            .await
            .map_err(internal)?;
        products.into_iter().map(ProductRow::into_product).collect()
    }

    async fn update_product(&self, id: Uuid, draft: &ProductDraft) -> Result<Product> {
        let nutrients = serde_json::to_string(&draft.nutrients).map_err(internal)?;
        let product: Option<ProductRow> = sqlx::query_as(UPDATE)
            .bind(&draft.name)
            .bind(draft.unit.as_str())
            .bind(draft.macros.energy_kcal)
            .bind(draft.macros.fat_g)
            .bind(draft.macros.protein_g)
            .bind(draft.macros.carbohydrates_g)
            .bind(nutrients)
            .bind(draft.cost)
            .bind(draft.brand.as_deref())
            .bind(row::now())
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        let Some(product) = product else {
            return Err(row::not_found(Entity::Product, id));
        };
        product.into_product()
    }

    async fn delete_product(&self, id: Uuid) -> Result<()> {
        let now = row::now();
        let deleted = sqlx::query(DELETE)
            .bind(&now)
            .bind(&now)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(internal)?;
        if deleted.rows_affected() == 0 {
            return Err(row::not_found(Entity::Product, id));
        }
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ProductRow {
    id: String,
    name: String,
    unit: String,
    energy_kcal: f64,
    fat_g: f64,
    protein_g: f64,
    carbohydrates_g: f64,
    nutrients: String,
    cost: Option<f64>,
    brand: Option<String>,
    source_code: Option<String>,
    created_at: String,
    updated_at: String,
}

impl ProductRow {
    fn into_product(self) -> Result<Product> {
        let Self {
            id,
            name,
            unit,
            energy_kcal,
            fat_g,
            protein_g,
            carbohydrates_g,
            nutrients,
            cost,
            brand,
            source_code,
            created_at,
            updated_at,
        } = self;

        let id = row::parse_id(&id)?;
        let unit = unit.parse().map_err(internal)?;
        let macros = Macros {
            energy_kcal,
            fat_g,
            protein_g,
            carbohydrates_g,
        };
        let nutrients = serde_json::from_str(&nutrients).map_err(internal)?;
        let created_at = row::parse_timestamp(&created_at)?;
        let updated_at = row::parse_timestamp(&updated_at)?;

        Ok(Product {
            id,
            name,
            unit,
            macros,
            nutrients,
            cost,
            brand,
            source_code,
            created_at,
            updated_at,
        })
    }
}
