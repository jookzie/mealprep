package product

import (
	"context"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/errorx"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

func (r Repository) Delete(ctx context.Context, id uuid.UUID) error {
	affected, err := r.queries.SoftDeleteProduct(ctx, db.SoftDeleteProductParams{
		DeletedAt: sqliterepo.NullNow(),
		UpdatedAt: sqliterepo.Now(),
		ID:        id.String(),
	})
	if err != nil {
		return err
	}
	if affected == 0 {
		return errorx.NotFound("product", id.String(), nil)
	}
	return nil
}
