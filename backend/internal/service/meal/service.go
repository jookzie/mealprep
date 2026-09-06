// Package meal holds the meal domain logic, including deriving a meal's macros.
package meal

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

type Repository interface {
	Create(ctx context.Context, meal domain.Meal) (domain.Meal, error)
	Get(ctx context.Context, id uuid.UUID) (domain.Meal, error)
	List(ctx context.Context) ([]domain.Meal, error)
	Update(ctx context.Context, meal domain.Meal) (domain.Meal, error)
	Delete(ctx context.Context, id uuid.UUID) error
}

type Products interface {
	ListByIDs(ctx context.Context, ids []uuid.UUID) ([]domain.Product, error)
}

type Service struct {
	repository Repository
	products   Products
}
