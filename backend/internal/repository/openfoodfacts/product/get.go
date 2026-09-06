package product

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
)

func (r Repository) Get(ctx context.Context, code string) (domain.CatalogEntry, error) {
	product, err := r.client.Get(ctx, code)
	if err != nil {
		return domain.CatalogEntry{}, err
	}
	return fromProduct(product), nil
}
