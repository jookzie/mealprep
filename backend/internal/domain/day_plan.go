package domain

import (
	"time"

	"github.com/google/uuid"
)

type DayPlan struct {
	ID      uuid.UUID
	Label   string
	MealIDs []uuid.UUID
	// Meals and Macros are derived from MealIDs.
	Meals     []Meal
	Macros    Macros
	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt *time.Time
}
