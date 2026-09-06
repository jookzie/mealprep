package product

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) List(ctx context.Context) ([]domain.Product, error) {
	return s.repository.List(ctx)
}
