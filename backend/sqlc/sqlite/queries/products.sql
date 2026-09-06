-- name: CreateProduct :one
INSERT INTO products (id, name, unit, energy_kcal, fat_g, protein_g, carbohydrates_g, nutrients, source_code, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
RETURNING *;

-- name: GetProduct :one
SELECT * FROM products WHERE id = ? AND deleted_at IS NULL;

-- name: ListProducts :many
SELECT * FROM products WHERE deleted_at IS NULL ORDER BY name, id;

-- name: ListProductsByIDs :many
SELECT * FROM products WHERE id IN (sqlc.slice('ids')) AND deleted_at IS NULL;

-- name: UpdateProduct :one
UPDATE products
SET name = ?, unit = ?, energy_kcal = ?, fat_g = ?, protein_g = ?, carbohydrates_g = ?, nutrients = ?, updated_at = ?
WHERE id = ? AND deleted_at IS NULL
RETURNING *;

-- name: SoftDeleteProduct :execrows
UPDATE products SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL;
