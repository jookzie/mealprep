package targets

import (
	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

func fromRow(row db.Target) (domain.Targets, error) {
	createdAt, err := sqliterepo.ParseTime(row.CreatedAt)
	if err != nil {
		return domain.Targets{}, err
	}
	updatedAt, err := sqliterepo.ParseTime(row.UpdatedAt)
	if err != nil {
		return domain.Targets{}, err
	}
	deletedAt, err := sqliterepo.ParseNullTime(row.DeletedAt)
	if err != nil {
		return domain.Targets{}, err
	}

	return domain.Targets{
		Macros: domain.Macros{
			EnergyKcal:     row.EnergyKcal,
			FatG:           row.FatG,
			ProteinG:       row.ProteinG,
			CarbohydratesG: row.CarbohydratesG,
		},
		CreatedAt: createdAt,
		UpdatedAt: updatedAt,
		DeletedAt: deletedAt,
	}, nil
}
