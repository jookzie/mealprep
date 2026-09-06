package product

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
)

func (r Repository) Search(ctx context.Context, query string) ([]domain.CatalogEntry, error) {
	products, err := r.client.Search(ctx, query)
	if err != nil {
		return nil, err
	}

	entries := make([]domain.CatalogEntry, 0, len(products))
	for _, product := range products {
		entries = append(entries, fromProduct(product))
	}
	return entries, nil
}
