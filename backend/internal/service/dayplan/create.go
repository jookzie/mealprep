package dayplan

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) Create(ctx context.Context, plan domain.DayPlan) (domain.DayPlan, error) {
	if err := validate(plan); err != nil {
		return domain.DayPlan{}, err
	}
	if _, err := s.resolve(ctx, []domain.DayPlan{plan}, true); err != nil {
		return domain.DayPlan{}, err
	}

	plan.ID = uuid.New()
	created, err := s.repository.Create(ctx, plan)
	if err != nil {
		return domain.DayPlan{}, err
	}
	return s.Get(ctx, created.ID)
}
