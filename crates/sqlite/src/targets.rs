use mealprep_core::{
    Result,
    domain::{Macros, Targets},
    store::TargetsStore,
};

use crate::{
    SqliteStore,
    row::{self, internal},
};

const SELECT: &str = "SELECT energy_kcal, fat_g, protein_g, carbohydrates_g, created_at, updated_at \
    FROM targets WHERE id = 1 AND deleted_at IS NULL";
const SET: &str = "INSERT INTO targets \
    (id, energy_kcal, fat_g, protein_g, carbohydrates_g, created_at, updated_at) \
    VALUES (1, ?, ?, ?, ?, ?, ?) \
    ON CONFLICT (id) DO UPDATE \
    SET energy_kcal = excluded.energy_kcal, fat_g = excluded.fat_g, protein_g = excluded.protein_g, \
        carbohydrates_g = excluded.carbohydrates_g, updated_at = excluded.updated_at, deleted_at = NULL \
    RETURNING energy_kcal, fat_g, protein_g, carbohydrates_g, created_at, updated_at";

impl TargetsStore for SqliteStore {
    async fn get_targets(&self) -> Result<Option<Targets>> {
        let targets: Option<TargetsRow> = sqlx::query_as(SELECT)
            .fetch_optional(&self.pool)
            .await
            .map_err(internal)?;
        targets.map(TargetsRow::into_targets).transpose()
    }

    async fn set_targets(&self, macros: Macros) -> Result<Targets> {
        let now = row::now();
        let targets: TargetsRow = sqlx::query_as(SET)
            .bind(macros.energy_kcal)
            .bind(macros.fat_g)
            .bind(macros.protein_g)
            .bind(macros.carbohydrates_g)
            .bind(&now)
            .bind(&now)
            .fetch_one(&self.pool)
            .await
            .map_err(internal)?;
        targets.into_targets()
    }
}

#[derive(sqlx::FromRow)]
struct TargetsRow {
    energy_kcal: f64,
    fat_g: f64,
    protein_g: f64,
    carbohydrates_g: f64,
    created_at: String,
    updated_at: String,
}

impl TargetsRow {
    fn into_targets(self) -> Result<Targets> {
        let Self {
            energy_kcal,
            fat_g,
            protein_g,
            carbohydrates_g,
            created_at,
            updated_at,
        } = self;

        let macros = Macros {
            energy_kcal,
            fat_g,
            protein_g,
            carbohydrates_g,
        };
        let created_at = row::parse_timestamp(&created_at)?;
        let updated_at = row::parse_timestamp(&updated_at)?;

        Ok(Targets {
            macros,
            created_at,
            updated_at,
        })
    }
}
