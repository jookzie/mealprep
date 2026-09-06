// Package calendar serves the calendar endpoints of the generated API.
package calendar

import (
	"context"
	"time"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

type Service interface {
	Assign(ctx context.Context, date time.Time, planID uuid.UUID) (domain.CalendarDay, error)
	List(ctx context.Context, from, to time.Time) ([]domain.CalendarDay, error)
	Unassign(ctx context.Context, date time.Time) error
}

type Handler struct {
	calendar Service
}
