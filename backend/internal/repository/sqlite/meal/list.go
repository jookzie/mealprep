package meal

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

func (r Repository) List(ctx context.Context) ([]domain.Meal, error) {
	rows, err := r.queries.ListMeals(ctx)
	if err != nil {
		return nil, err
	}
	return r.withServings(ctx, rows)
}

func (r Repository) ListByIDs(ctx context.Context, ids []uuid.UUID) ([]domain.Meal, error) {
	if len(ids) == 0 {
		return []domain.Meal{}, nil
	}
	rows, err := r.queries.ListMealsByIDs(ctx, sqliterepo.Strings(ids))
	if err != nil {
		return nil, err
	}
	return r.withServings(ctx, rows)
}

func (r Repository) withServings(ctx context.Context, rows []db.Meal) ([]domain.Meal, error) {
	if len(rows) == 0 {
		return []domain.Meal{}, nil
	}

	ids := make([]string, 0, len(rows))
	for _, row := range rows {
		ids = append(ids, row.ID)
	}

	servings, err := r.queries.ListMealServingsByMealIDs(ctx, ids)
	if err != nil {
		return nil, err
	}

	return fromRows(rows, servings)
}
