package product

import (
	"encoding/json"
	"errors"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

var (
	ErrInvalidID       = errors.New("invalid product id in database")
	ErrDecodeNutrients = errors.New("decode product nutrients")
	ErrEncodeNutrients = errors.New("encode product nutrients")
)

func fromRow(row db.Product) (domain.Product, error) {
	id, err := uuid.Parse(row.ID)
	if err != nil {
		return domain.Product{}, errors.Join(ErrInvalidID, err)
	}

	var nutrients map[string]float64
	if err = json.Unmarshal([]byte(row.Nutrients), &nutrients); err != nil {
		return domain.Product{}, errors.Join(ErrDecodeNutrients, err)
	}

	createdAt, err := sqliterepo.ParseTime(row.CreatedAt)
	if err != nil {
		return domain.Product{}, err
	}
	updatedAt, err := sqliterepo.ParseTime(row.UpdatedAt)
	if err != nil {
		return domain.Product{}, err
	}
	deletedAt, err := sqliterepo.ParseNullTime(row.DeletedAt)
	if err != nil {
		return domain.Product{}, err
	}

	return domain.Product{
		ID:   id,
		Name: row.Name,
		Unit: domain.Unit(row.Unit),
		Macros: domain.Macros{
			EnergyKcal:     row.EnergyKcal,
			FatG:           row.FatG,
			ProteinG:       row.ProteinG,
			CarbohydratesG: row.CarbohydratesG,
		},
		Nutrients:  nutrients,
		Brand:      row.Brand.String,
		SourceCode: row.SourceCode.String,
		CreatedAt:  createdAt,
		UpdatedAt:  updatedAt,
		DeletedAt:  deletedAt,
	}, nil
}

func fromRows(rows []db.Product) ([]domain.Product, error) {
	products := make([]domain.Product, 0, len(rows))
	for _, row := range rows {
		product, err := fromRow(row)
		if err != nil {
			return nil, err
		}
		products = append(products, product)
	}
	return products, nil
}

func encodeNutrients(nutrients map[string]float64) (string, error) {
	if nutrients == nil {
		nutrients = map[string]float64{}
	}
	encoded, err := json.Marshal(nutrients)
	if err != nil {
		return "", errors.Join(ErrEncodeNutrients, err)
	}
	return string(encoded), nil
}
