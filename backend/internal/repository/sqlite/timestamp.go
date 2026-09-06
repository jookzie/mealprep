package sqlite

import (
	"database/sql"
	"errors"
	"time"
)

var ErrInvalidTimestamp = errors.New("invalid timestamp in database")

// Timestamps are stored as RFC 3339 text in UTC.
func Now() string {
	return time.Now().UTC().Format(time.RFC3339Nano)
}

func NullNow() sql.NullString {
	return sql.NullString{String: Now(), Valid: true}
}

func ParseTime(value string) (time.Time, error) {
	parsed, err := time.Parse(time.RFC3339Nano, value)
	if err != nil {
		return time.Time{}, errors.Join(ErrInvalidTimestamp, err)
	}
	return parsed, nil
}

func ParseNullTime(value sql.NullString) (*time.Time, error) {
	if !value.Valid {
		return nil, nil
	}
	parsed, err := ParseTime(value.String)
	if err != nil {
		return nil, err
	}
	return &parsed, nil
}
