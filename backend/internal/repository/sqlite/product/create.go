package product

import (
	"context"
	"database/sql"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

func (r Repository) Create(ctx context.Context, product domain.Product) (domain.Product, error) {
	nutrients, err := encodeNutrients(product.Nutrients)
	if err != nil {
		return domain.Product{}, err
	}

	now := sqliterepo.Now()
	row, err := r.queries.CreateProduct(ctx, db.CreateProductParams{
		ID:             product.ID.String(),
		Name:           product.Name,
		Unit:           string(product.Unit),
		EnergyKcal:     product.Macros.EnergyKcal,
		FatG:           product.Macros.FatG,
		ProteinG:       product.Macros.ProteinG,
		CarbohydratesG: product.Macros.CarbohydratesG,
		Nutrients:      nutrients,
		Brand:          sql.NullString{String: product.Brand, Valid: product.Brand != ""},
		SourceCode:     sql.NullString{String: product.SourceCode, Valid: product.SourceCode != ""},
		CreatedAt:      now,
		UpdatedAt:      now,
	})
	if err != nil {
		return domain.Product{}, err
	}

	return fromRow(row)
}
