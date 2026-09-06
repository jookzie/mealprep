package targets

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

// Set creates the single row or replaces its figures.
func (r Repository) Set(ctx context.Context, macros domain.Macros) (domain.Targets, error) {
	now := sqliterepo.Now()
	row, err := r.queries.SetTargets(ctx, db.SetTargetsParams{
		EnergyKcal:     macros.EnergyKcal,
		FatG:           macros.FatG,
		ProteinG:       macros.ProteinG,
		CarbohydratesG: macros.CarbohydratesG,
		CreatedAt:      now,
		UpdatedAt:      now,
	})
	if err != nil {
		return domain.Targets{}, err
	}
	return fromRow(row)
}
