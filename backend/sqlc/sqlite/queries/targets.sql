-- name: SetTargets :one
INSERT INTO targets (id, energy_kcal, fat_g, protein_g, carbohydrates_g, created_at, updated_at)
VALUES (1, ?, ?, ?, ?, ?, ?)
ON CONFLICT (id) DO UPDATE
SET energy_kcal = excluded.energy_kcal, fat_g = excluded.fat_g, protein_g = excluded.protein_g,
    carbohydrates_g = excluded.carbohydrates_g, updated_at = excluded.updated_at, deleted_at = NULL
RETURNING *;

-- name: GetTargets :one
SELECT * FROM targets WHERE id = 1 AND deleted_at IS NULL;
