package meal

import (
	"context"
	"database/sql"
	"errors"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
)

func (r Repository) Get(ctx context.Context, id uuid.UUID) (domain.Meal, error) {
	row, err := r.queries.GetMeal(ctx, id.String())
	if errors.Is(err, sql.ErrNoRows) {
		return domain.Meal{}, errorx.NotFound("meal", id.String(), err)
	}
	if err != nil {
		return domain.Meal{}, err
	}

	servings, err := r.queries.ListMealServingsByMealIDs(ctx, []string{row.ID})
	if err != nil {
		return domain.Meal{}, err
	}

	return fromRow(row, servings)
}
