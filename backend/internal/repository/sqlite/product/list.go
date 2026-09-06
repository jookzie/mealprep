package product

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
)

func (r Repository) List(ctx context.Context) ([]domain.Product, error) {
	rows, err := r.queries.ListProducts(ctx)
	if err != nil {
		return nil, err
	}
	return fromRows(rows)
}

func (r Repository) ListByIDs(ctx context.Context, ids []uuid.UUID) ([]domain.Product, error) {
	if len(ids) == 0 {
		return []domain.Product{}, nil
	}
	rows, err := r.queries.ListProductsByIDs(ctx, sqliterepo.Strings(ids))
	if err != nil {
		return nil, err
	}
	return fromRows(rows)
}
