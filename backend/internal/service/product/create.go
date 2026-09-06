package product

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

// Create stores a product the user entered by hand. SourceCode is ignored;
// only Import sets it.
func (s Service) Create(ctx context.Context, product domain.Product) (domain.Product, error) {
	if err := validate(product); err != nil {
		return domain.Product{}, err
	}
	product.ID = uuid.New()
	product.SourceCode = ""
	return s.repository.Create(ctx, product)
}
