package meal

import (
	"context"
	"database/sql"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/errorx"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

func (r Repository) Delete(ctx context.Context, id uuid.UUID) error {
	now := sqliterepo.Now()

	return sqliterepo.Transact(ctx, r.conn, func(queries *db.Queries) error {
		affected, err := queries.SoftDeleteMeal(ctx, db.SoftDeleteMealParams{
			DeletedAt: sql.NullString{String: now, Valid: true},
			UpdatedAt: now,
			ID:        id.String(),
		})
		if err != nil {
			return err
		}
		if affected == 0 {
			return errorx.NotFound("meal", id.String(), nil)
		}

		return queries.SoftDeleteMealServings(ctx, db.SoftDeleteMealServingsParams{
			DeletedAt: sql.NullString{String: now, Valid: true},
			UpdatedAt: now,
			MealID:    id.String(),
		})
	})
}
