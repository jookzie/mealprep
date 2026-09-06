package domain

import (
	"time"

	"github.com/google/uuid"
)

// Serving is a product in a meal; Amount is in the product's Unit.
type Serving struct {
	ProductID uuid.UUID
	Amount    float64
}

type Meal struct {
	ID       uuid.UUID
	Label    string
	Servings []Serving
	// Macros is derived from the servings.
	Macros    Macros
	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt *time.Time
}
