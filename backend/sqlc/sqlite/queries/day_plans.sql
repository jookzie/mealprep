-- name: CreateDayPlan :one
INSERT INTO day_plans (id, label, created_at, updated_at) VALUES (?, ?, ?, ?) RETURNING *;

-- name: GetDayPlan :one
SELECT * FROM day_plans WHERE id = ? AND deleted_at IS NULL;

-- name: ListDayPlans :many
SELECT * FROM day_plans WHERE deleted_at IS NULL ORDER BY label, id;

-- name: ListDayPlansByIDs :many
SELECT * FROM day_plans WHERE id IN (sqlc.slice('ids')) AND deleted_at IS NULL;

-- name: UpdateDayPlan :one
UPDATE day_plans SET label = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL RETURNING *;

-- name: SoftDeleteDayPlan :execrows
UPDATE day_plans SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL;

-- name: CreateDayPlanMeal :exec
INSERT INTO day_plan_meals (id, day_plan_id, meal_id, position, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?);

-- name: ListDayPlanMealsByDayPlanIDs :many
SELECT * FROM day_plan_meals
WHERE day_plan_id IN (sqlc.slice('day_plan_ids')) AND deleted_at IS NULL
ORDER BY day_plan_id, position;

-- name: SoftDeleteDayPlanMeals :exec
UPDATE day_plan_meals SET deleted_at = ?, updated_at = ? WHERE day_plan_id = ? AND deleted_at IS NULL;
