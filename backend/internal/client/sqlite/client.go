// Package sqlite opens the SQLite database and applies the schema.
package sqlite

import (
	"database/sql"

	_ "modernc.org/sqlite" // registers the "sqlite" driver
)

// Client owns the connection pool. Repositories run their queries through DB.
type Client struct {
	DB *sql.DB
}

func (c Client) Close() error {
	return c.DB.Close()
}
