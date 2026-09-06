// Package product holds the product domain logic: the user's own products and
// the catalog they can be imported from.
package product

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

type Repository interface {
	Create(ctx context.Context, product domain.Product) (domain.Product, error)
	Get(ctx context.Context, id uuid.UUID) (domain.Product, error)
	List(ctx context.Context) ([]domain.Product, error)
	Update(ctx context.Context, product domain.Product) (domain.Product, error)
	Delete(ctx context.Context, id uuid.UUID) error
}

type Catalog interface {
	Search(ctx context.Context, query string) ([]domain.CatalogEntry, error)
	Get(ctx context.Context, code string) (domain.CatalogEntry, error)
}

type Service struct {
	repository Repository
	catalog    Catalog
}
