package product

import (
	"context"
	"database/sql"
	"errors"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

func (r Repository) Update(ctx context.Context, product domain.Product) (domain.Product, error) {
	nutrients, err := encodeNutrients(product.Nutrients)
	if err != nil {
		return domain.Product{}, err
	}

	row, err := r.queries.UpdateProduct(ctx, db.UpdateProductParams{
		Name:           product.Name,
		Unit:           string(product.Unit),
		EnergyKcal:     product.Macros.EnergyKcal,
		FatG:           product.Macros.FatG,
		ProteinG:       product.Macros.ProteinG,
		CarbohydratesG: product.Macros.CarbohydratesG,
		Nutrients:      nutrients,
		Brand:          sql.NullString{String: product.Brand, Valid: product.Brand != ""},
		UpdatedAt:      sqliterepo.Now(),
		ID:             product.ID.String(),
	})
	if errors.Is(err, sql.ErrNoRows) {
		return domain.Product{}, errorx.NotFound("product", product.ID.String(), err)
	}
	if err != nil {
		return domain.Product{}, err
	}

	return fromRow(row)
}
