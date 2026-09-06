// Package dayplan holds the day plan domain logic, including summing its meals.
package dayplan

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

type Repository interface {
	Create(ctx context.Context, plan domain.DayPlan) (domain.DayPlan, error)
	Get(ctx context.Context, id uuid.UUID) (domain.DayPlan, error)
	List(ctx context.Context) ([]domain.DayPlan, error)
	Update(ctx context.Context, plan domain.DayPlan) (domain.DayPlan, error)
	Delete(ctx context.Context, id uuid.UUID) error
}

type Meals interface {
	ListByIDs(ctx context.Context, ids []uuid.UUID) ([]domain.Meal, error)
}

type Products interface {
	ListByIDs(ctx context.Context, ids []uuid.UUID) ([]domain.Product, error)
}

type Service struct {
	repository Repository
	meals      Meals
	products   Products
}
