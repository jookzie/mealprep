package dayplan

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

func (r Repository) Create(ctx context.Context, plan domain.DayPlan) (domain.DayPlan, error) {
	now := sqliterepo.Now()

	err := sqliterepo.Transact(ctx, r.conn, func(queries *db.Queries) error {
		_, err := queries.CreateDayPlan(ctx, db.CreateDayPlanParams{
			ID:        plan.ID.String(),
			Label:     plan.Label,
			CreatedAt: now,
			UpdatedAt: now,
		})
		if err != nil {
			return err
		}
		return insertMeals(ctx, queries, plan, now)
	})
	if err != nil {
		return domain.DayPlan{}, err
	}

	return r.Get(ctx, plan.ID)
}

func insertMeals(ctx context.Context, queries *db.Queries, plan domain.DayPlan, now string) error {
	for position, mealID := range plan.MealIDs {
		err := queries.CreateDayPlanMeal(ctx, db.CreateDayPlanMealParams{
			ID:        uuid.NewString(),
			DayPlanID: plan.ID.String(),
			MealID:    mealID.String(),
			Position:  int64(position),
			CreatedAt: now,
			UpdatedAt: now,
		})
		if err != nil {
			return err
		}
	}
	return nil
}
