package product

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) Get(ctx context.Context, id uuid.UUID) (domain.Product, error) {
	return s.repository.Get(ctx, id)
}
