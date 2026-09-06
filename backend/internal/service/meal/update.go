package meal

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) Update(ctx context.Context, meal domain.Meal) (domain.Meal, error) {
	if err := validate(meal); err != nil {
		return domain.Meal{}, err
	}
	if _, err := s.resolve(ctx, []domain.Meal{meal}, true); err != nil {
		return domain.Meal{}, err
	}

	updated, err := s.repository.Update(ctx, meal)
	if err != nil {
		return domain.Meal{}, err
	}
	return s.Get(ctx, updated.ID)
}
