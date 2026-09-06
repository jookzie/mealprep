package meal

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) List(ctx context.Context) ([]domain.Meal, error) {
	meals, err := s.repository.List(ctx)
	if err != nil {
		return nil, err
	}
	return s.derive(ctx, meals)
}
