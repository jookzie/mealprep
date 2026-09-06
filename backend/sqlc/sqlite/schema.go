// Package sqlite holds the SQLite schema and queries sqlc generates from, and
// exposes the schema so the client can apply it at startup.
package sqlite

import _ "embed"

//go:embed schema.sql
var Schema string
