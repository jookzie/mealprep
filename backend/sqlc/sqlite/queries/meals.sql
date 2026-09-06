-- name: CreateMeal :one
INSERT INTO meals (id, label, created_at, updated_at) VALUES (?, ?, ?, ?) RETURNING *;

-- name: GetMeal :one
SELECT * FROM meals WHERE id = ? AND deleted_at IS NULL;

-- name: ListMeals :many
SELECT * FROM meals WHERE deleted_at IS NULL ORDER BY label, id;

-- name: ListMealsByIDs :many
SELECT * FROM meals WHERE id IN (sqlc.slice('ids')) AND deleted_at IS NULL;

-- name: UpdateMeal :one
UPDATE meals SET label = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL RETURNING *;

-- name: SoftDeleteMeal :execrows
UPDATE meals SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL;

-- name: CreateMealServing :exec
INSERT INTO meal_servings (id, meal_id, product_id, amount, position, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?);

-- name: ListMealServingsByMealIDs :many
SELECT * FROM meal_servings
WHERE meal_id IN (sqlc.slice('meal_ids')) AND deleted_at IS NULL
ORDER BY meal_id, position;

-- name: SoftDeleteMealServings :exec
UPDATE meal_servings SET deleted_at = ?, updated_at = ? WHERE meal_id = ? AND deleted_at IS NULL;
