package meal

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

func (r Repository) Create(ctx context.Context, meal domain.Meal) (domain.Meal, error) {
	now := sqliterepo.Now()

	err := sqliterepo.Transact(ctx, r.conn, func(queries *db.Queries) error {
		_, err := queries.CreateMeal(ctx, db.CreateMealParams{
			ID:        meal.ID.String(),
			Label:     meal.Label,
			CreatedAt: now,
			UpdatedAt: now,
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

func insertServings(ctx context.Context, queries *db.Queries, meal domain.Meal, now string) error {
	for position, serving := range meal.Servings {
		err := queries.CreateMealServing(ctx, db.CreateMealServingParams{
			ID:        uuid.NewString(),
			MealID:    meal.ID.String(),
			ProductID: serving.ProductID.String(),
			Amount:    serving.Amount,
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
