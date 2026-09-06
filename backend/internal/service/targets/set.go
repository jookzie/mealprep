package targets

import (
	"context"
	"errors"
	"fmt"

	"github.com/mertan/mealprep/internal/domain"
)

var ErrInvalidTargets = errors.New("invalid targets")

func (s Service) Set(ctx context.Context, macros domain.Macros) (domain.Targets, error) {
	var causes []error
	for name, value := range map[string]float64{
		"energy":        macros.EnergyKcal,
		"fat":           macros.FatG,
		"protein":       macros.ProteinG,
		"carbohydrates": macros.CarbohydratesG,
	} {
		if value < 0 {
			causes = append(causes, fmt.Errorf("%s is negative", name))
		}
	}
	if len(causes) > 0 {
		return domain.Targets{}, errors.Join(append([]error{ErrInvalidTargets}, causes...)...)
	}

	return s.repository.Set(ctx, macros)
}
