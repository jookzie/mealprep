// Package product persists products in SQLite.
package product

import (
	"database/sql"

	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

type Repository struct {
	conn    *sql.DB
	queries *db.Queries
}
