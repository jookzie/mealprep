// Package meal holds the meal domain logic.
package meal

import (
	"sync"

	"github.com/google/uuid"
)

type Meal struct {
	ID       uuid.UUID
	Label    string
	Calories int
}

// Service keeps meals in memory until a repository exists, so it carries a mutex
// and is used through a pointer.
type Service struct {
	mu    sync.RWMutex
	meals []Meal
}
