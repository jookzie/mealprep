// Package calendar persists day plan assignments to dates in SQLite.
package calendar

import (
	"database/sql"

	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

// Dates are stored as YYYY-MM-DD, which sorts and compares as text.
const dateLayout = "2006-01-02"

type Repository struct {
	conn    *sql.DB
	queries *db.Queries
}
