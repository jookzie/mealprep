// Package meal serves the meal endpoints of the generated API.
package meal

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

type Service interface {
	List(ctx context.Context) ([]domain.Meal, error)
	Get(ctx context.Context, id uuid.UUID) (domain.Meal, error)
	Create(ctx context.Context, meal domain.Meal) (domain.Meal, error)
	Update(ctx context.Context, meal domain.Meal) (domain.Meal, error)
	Delete(ctx context.Context, id uuid.UUID) error
}

type Handler struct {
	meals Service
}
