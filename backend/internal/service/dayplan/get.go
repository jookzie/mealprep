package dayplan

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) Get(ctx context.Context, id uuid.UUID) (domain.DayPlan, error) {
	plan, err := s.repository.Get(ctx, id)
	if err != nil {
		return domain.DayPlan{}, err
	}
	derived, err := s.derive(ctx, []domain.DayPlan{plan})
	if err != nil {
		return domain.DayPlan{}, err
	}
	return derived[0], nil
}
