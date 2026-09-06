package meal

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) Get(ctx context.Context, id uuid.UUID) (domain.Meal, error) {
	meal, err := s.repository.Get(ctx, id)
	if err != nil {
		return domain.Meal{}, err
	}
	derived, err := s.derive(ctx, []domain.Meal{meal})
	if err != nil {
		return domain.Meal{}, err
	}
	return derived[0], nil
}
