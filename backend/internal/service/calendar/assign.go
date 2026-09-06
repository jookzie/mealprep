package calendar

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
)

var ErrUnknownDayPlan = errors.New("calendar day refers to a day plan that does not exist")

// Assign puts a day plan on a date, replacing whatever was there.
func (s Service) Assign(ctx context.Context, date time.Time, planID uuid.UUID) (domain.CalendarDay, error) {
	var notFound errorx.ErrNotFound
	_, err := s.dayPlans.Get(ctx, planID)
	if errors.As(err, &notFound) {
		return domain.CalendarDay{}, errors.Join(ErrUnknownDayPlan, err)
	}
	if err != nil {
		return domain.CalendarDay{}, err
	}

	return s.repository.Assign(ctx, date, planID)
}
