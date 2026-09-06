package sqlite

import (
	"context"
	"database/sql"
	"errors"

	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

// Transact runs fn inside one transaction, rolling back on any error.
func Transact(ctx context.Context, conn *sql.DB, fn func(queries *db.Queries) error) error {
	tx, err := conn.BeginTx(ctx, nil)
	if err != nil {
		return err
	}

	if err = fn(db.New(tx)); err != nil {
		return errors.Join(err, tx.Rollback())
	}

	return tx.Commit()
}
