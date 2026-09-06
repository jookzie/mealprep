package product

import (
	"context"
	"database/sql"
	"errors"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
)

func (r Repository) Get(ctx context.Context, id uuid.UUID) (domain.Product, error) {
	row, err := r.queries.GetProduct(ctx, id.String())
	if errors.Is(err, sql.ErrNoRows) {
		return domain.Product{}, errorx.NotFound("product", id.String(), err)
	}
	if err != nil {
		return domain.Product{}, err
	}

	return fromRow(row)
}
