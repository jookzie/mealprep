// Package targets persists the single user's daily targets in SQLite.
package targets

import (
	"database/sql"

	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

type Repository struct {
	conn    *sql.DB
	queries *db.Queries
}
