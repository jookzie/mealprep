// Package meal persists meals and their servings in SQLite.
package meal

import (
	"database/sql"

	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

type Repository struct {
	conn    *sql.DB
	queries *db.Queries
}
