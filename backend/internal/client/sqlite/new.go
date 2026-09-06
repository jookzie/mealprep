package sqlite

import (
	"database/sql"
	"errors"

	"github.com/mertan/mealprep/sqlc/sqlite"
)

var (
	ErrEmptyPath = errors.New("sqlite path is empty")
	ErrOpen      = errors.New("open sqlite database")
	ErrSchema    = errors.New("apply sqlite schema")
)

func New(path string) (Client, error) {
	if path == "" {
		return Client{}, ErrEmptyPath
	}

	// A single writer keeps SQLite's locking out of the way; foreign keys are off by default.
	db, err := sql.Open("sqlite", path+"?_pragma=foreign_keys(1)&_pragma=busy_timeout(5000)&_pragma=journal_mode(WAL)")
	if err != nil {
		return Client{}, errors.Join(ErrOpen, err)
	}
	db.SetMaxOpenConns(1)

	if _, err = db.Exec(sqlite.Schema); err != nil {
		return Client{}, errors.Join(ErrSchema, err, db.Close())
	}

	return Client{DB: db}, nil
}
