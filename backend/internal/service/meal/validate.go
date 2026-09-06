package meal

import (
	"errors"
	"fmt"

	"github.com/mertan/mealprep/internal/domain"
)

var ErrInvalidMeal = errors.New("invalid meal")

func validate(meal domain.Meal) error {
	var causes []error

	if meal.Label == "" {
		causes = append(causes, errors.New("label is empty"))
	}
	for i, serving := range meal.Servings {
		if serving.Amount <= 0 {
			causes = append(causes, fmt.Errorf("serving %d amount must be positive", i))
		}
	}

	if len(causes) == 0 {
		return nil
	}
	return errors.Join(append([]error{ErrInvalidMeal}, causes...)...)
}
