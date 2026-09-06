package dayplan

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

func (r Repository) List(ctx context.Context) ([]domain.DayPlan, error) {
	rows, err := r.queries.ListDayPlans(ctx)
	if err != nil {
		return nil, err
	}
	return r.withMeals(ctx, rows)
}

func (r Repository) ListByIDs(ctx context.Context, ids []uuid.UUID) ([]domain.DayPlan, error) {
	if len(ids) == 0 {
		return []domain.DayPlan{}, nil
	}
	rows, err := r.queries.ListDayPlansByIDs(ctx, sqliterepo.Strings(ids))
	if err != nil {
		return nil, err
	}
	return r.withMeals(ctx, rows)
}

func (r Repository) withMeals(ctx context.Context, rows []db.DayPlan) ([]domain.DayPlan, error) {
	if len(rows) == 0 {
		return []domain.DayPlan{}, nil
	}

	ids := make([]string, 0, len(rows))
	for _, row := range rows {
		ids = append(ids, row.ID)
	}

	meals, err := r.queries.ListDayPlanMealsByDayPlanIDs(ctx, ids)
	if err != nil {
		return nil, err
	}

	return fromRows(rows, meals)
}
