// Package dayplan persists day plans and their meals in SQLite.
package dayplan

import (
	"database/sql"

	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

type Repository struct {
	conn    *sql.DB
	queries *db.Queries
}
