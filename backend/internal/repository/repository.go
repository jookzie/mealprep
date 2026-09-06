package repository

// Repository is the persistence boundary. SQLite is the first implementation.
type Repository interface{}

// NewSQLite opens a SQLite-backed Repository.
func NewSQLite(path string) (Repository, error) {
	panic("Not implemented yet")
}
