// Package calendar holds the calendar domain logic: one day plan per date.
package calendar

import (
	"context"
	"time"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

type Repository interface {
	Assign(ctx context.Context, date time.Time, planID uuid.UUID) (domain.CalendarDay, error)
	Get(ctx context.Context, date time.Time) (domain.CalendarDay, error)
	List(ctx context.Context, from, to time.Time) ([]domain.CalendarDay, error)
	Unassign(ctx context.Context, date time.Time) error
}

type DayPlans interface {
	Get(ctx context.Context, id uuid.UUID) (domain.DayPlan, error)
}

type Service struct {
	repository Repository
	dayPlans   DayPlans
}
