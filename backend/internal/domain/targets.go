package domain

import "time"

// Targets are the single user's daily figures; there is exactly one row.
type Targets struct {
	Macros    Macros
	CreatedAt time.Time
	UpdatedAt time.Time
	DeletedAt *time.Time
}
