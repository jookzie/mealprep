// Package product serves the product endpoints of the generated API.
package product

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

type Service interface {
	List(ctx context.Context) ([]domain.Product, error)
	Get(ctx context.Context, id uuid.UUID) (domain.Product, error)
	Create(ctx context.Context, product domain.Product) (domain.Product, error)
	Update(ctx context.Context, product domain.Product) (domain.Product, error)
	Delete(ctx context.Context, id uuid.UUID) error
	Search(ctx context.Context, query string) ([]domain.CatalogEntry, error)
	Import(ctx context.Context, code string) (domain.Product, error)
}

type Handler struct {
	products Service
}
