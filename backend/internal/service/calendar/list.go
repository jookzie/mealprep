package calendar

import (
	"context"
	"errors"
	"time"

	"github.com/mertan/mealprep/internal/domain"
)

var ErrInvalidRange = errors.New("invalid date range")

// Longer ranges are a bulk export, not a calendar view.
const maxRangeDays = 366

func (s Service) List(ctx context.Context, from, to time.Time) ([]domain.CalendarDay, error) {
	if to.Before(from) {
		return nil, errors.Join(ErrInvalidRange, errors.New("to is before from"))
	}
	if to.Sub(from) > maxRangeDays*24*time.Hour {
		return nil, errors.Join(ErrInvalidRange, errors.New("range exceeds one year"))
	}
	return s.repository.List(ctx, from, to)
}
