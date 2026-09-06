package dayplan

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) Update(ctx context.Context, plan domain.DayPlan) (domain.DayPlan, error) {
	if err := validate(plan); err != nil {
		return domain.DayPlan{}, err
	}
	if _, err := s.resolve(ctx, []domain.DayPlan{plan}, true); err != nil {
		return domain.DayPlan{}, err
	}

	updated, err := s.repository.Update(ctx, plan)
	if err != nil {
		return domain.DayPlan{}, err
	}
	return s.Get(ctx, updated.ID)
}
