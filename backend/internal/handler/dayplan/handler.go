// Package dayplan serves the day plan endpoints of the generated API.
package dayplan

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

type Service interface {
	List(ctx context.Context) ([]domain.DayPlan, error)
	Get(ctx context.Context, id uuid.UUID) (domain.DayPlan, error)
	Create(ctx context.Context, plan domain.DayPlan) (domain.DayPlan, error)
	Update(ctx context.Context, plan domain.DayPlan) (domain.DayPlan, error)
	Delete(ctx context.Context, id uuid.UUID) error
}

type Handler struct {
	dayPlans Service
}
