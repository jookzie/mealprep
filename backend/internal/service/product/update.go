package product

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) Update(ctx context.Context, product domain.Product) (domain.Product, error) {
	if err := validate(product); err != nil {
		return domain.Product{}, err
	}
	return s.repository.Update(ctx, product)
}
