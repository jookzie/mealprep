package dayplan

import (
	"errors"

	"github.com/mertan/mealprep/internal/domain"
)

var ErrInvalidDayPlan = errors.New("invalid day plan")

func validate(plan domain.DayPlan) error {
	if plan.Label == "" {
		return errors.Join(ErrInvalidDayPlan, errors.New("label is empty"))
	}
	return nil
}
