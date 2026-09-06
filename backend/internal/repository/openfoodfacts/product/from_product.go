package product

import (
	"github.com/mertan/mealprep/internal/client/openfoodfacts"
	"github.com/mertan/mealprep/internal/domain"
)

// The four macros under the names Open Food Facts uses.
const (
	keyEnergy        = "energy-kcal"
	keyFat           = "fat"
	keyProtein       = "proteins"
	keyCarbohydrates = "carbohydrates"
)

func fromProduct(product openfoodfacts.Product) domain.CatalogEntry {
	energy, hasEnergy := product.Nutriments[keyEnergy]
	fat, hasFat := product.Nutriments[keyFat]
	protein, hasProtein := product.Nutriments[keyProtein]
	carbohydrates, hasCarbohydrates := product.Nutriments[keyCarbohydrates]

	nutrients := make(map[string]float64, len(product.Nutriments))
	for name, value := range product.Nutriments {
		switch name {
		case keyEnergy, keyFat, keyProtein, keyCarbohydrates:
			continue
		}
		nutrients[name] = value
	}

	unit := domain.Gram
	if product.NutritionDataPer == "100ml" {
		unit = domain.Millilitre
	}

	return domain.CatalogEntry{
		Code: product.Code,
		Name: product.Name,
		Unit: unit,
		Macros: domain.Macros{
			EnergyKcal:     energy,
			FatG:           fat,
			ProteinG:       protein,
			CarbohydratesG: carbohydrates,
		},
		Nutrients: nutrients,
		Complete:  hasEnergy && hasFat && hasProtein && hasCarbohydrates,
	}
}
