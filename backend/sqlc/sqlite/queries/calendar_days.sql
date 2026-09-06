-- name: AssignCalendarDay :one
INSERT INTO calendar_days (date, day_plan_id, created_at, updated_at)
VALUES (?, ?, ?, ?)
ON CONFLICT (date) DO UPDATE
SET day_plan_id = excluded.day_plan_id, updated_at = excluded.updated_at, deleted_at = NULL
RETURNING *;

-- name: GetCalendarDay :one
SELECT * FROM calendar_days WHERE date = ? AND deleted_at IS NULL;

-- name: ListCalendarDays :many
SELECT * FROM calendar_days WHERE date >= ? AND date <= ? AND deleted_at IS NULL ORDER BY date;

-- name: SoftDeleteCalendarDay :execrows
UPDATE calendar_days SET deleted_at = ?, updated_at = ? WHERE date = ? AND deleted_at IS NULL;
