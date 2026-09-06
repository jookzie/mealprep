package meal

import (
	"context"
	"database/sql"
	"errors"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

// Update replaces the label and the whole serving list; old servings are soft-deleted.
func (r Repository) Update(ctx context.Context, meal domain.Meal) (domain.Meal, error) {
	now := sqliterepo.Now()

	err := sqliterepo.Transact(ctx, r.conn, func(queries *db.Queries) error {
		_, err := queries.UpdateMeal(ctx, db.UpdateMealParams{
			Label:     meal.Label,
			UpdatedAt: now,
			ID:        meal.ID.String(),
		})
		if errors.Is(err, sql.ErrNoRows) {
			return errorx.NotFound("meal", meal.ID.String(), err)
		}
		if err != nil {
			return err
		}

		err = queries.SoftDeleteMealServings(ctx, db.SoftDeleteMealServingsParams{
			DeletedAt: sql.NullString{String: now, Valid: true},
			UpdatedAt: now,
			MealID:    meal.ID.String(),
		})
		if err != nil {
			return err
		}

		return insertServings(ctx, queries, meal, now)
	})
	if err != nil {
		return domain.Meal{}, err
	}

	return r.Get(ctx, meal.ID)
}
