package product

import (
	"errors"
	"fmt"

	"github.com/mertan/mealprep/internal/domain"
)

var ErrInvalidProduct = errors.New("invalid product")

func validate(product domain.Product) error {
	var causes []error

	if product.Name == "" {
		causes = append(causes, errors.New("name is empty"))
	}
	if product.Unit != domain.Gram && product.Unit != domain.Millilitre {
		causes = append(causes, fmt.Errorf("unit must be %q or %q", domain.Gram, domain.Millilitre))
	}
	for name, value := range map[string]float64{
		"energy":        product.Macros.EnergyKcal,
		"fat":           product.Macros.FatG,
		"protein":       product.Macros.ProteinG,
		"carbohydrates": product.Macros.CarbohydratesG,
	} {
		if value < 0 {
			causes = append(causes, fmt.Errorf("%s is negative", name))
		}
	}
	for name, value := range product.Nutrients {
		if value < 0 {
			causes = append(causes, fmt.Errorf("nutrient %q is negative", name))
		}
	}

	if len(causes) == 0 {
		return nil
	}
	return errors.Join(append([]error{ErrInvalidProduct}, causes...)...)
}
