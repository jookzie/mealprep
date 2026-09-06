package dayplan

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) List(ctx context.Context) ([]domain.DayPlan, error) {
	plans, err := s.repository.List(ctx)
	if err != nil {
		return nil, err
	}
	return s.derive(ctx, plans)
}
