package meal

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) Create(ctx context.Context, meal domain.Meal) (domain.Meal, error) {
	if err := validate(meal); err != nil {
		return domain.Meal{}, err
	}
	if _, err := s.resolve(ctx, []domain.Meal{meal}, true); err != nil {
		return domain.Meal{}, err
	}

	meal.ID = uuid.New()
	created, err := s.repository.Create(ctx, meal)
	if err != nil {
		return domain.Meal{}, err
	}
	return s.Get(ctx, created.ID)
}
