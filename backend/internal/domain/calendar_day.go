package domain

import (
	"time"

	"github.com/google/uuid"
)

// CalendarDay assigns one day plan to one date. Date carries no time of day.
type CalendarDay struct {
	Date      time.Time
	DayPlanID uuid.UUID
	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt *time.Time
}
