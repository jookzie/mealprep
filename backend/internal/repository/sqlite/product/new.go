package product

import (
	"errors"

	sqliteclient "github.com/mertan/mealprep/internal/client/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

var ErrNilClient = errors.New("sqlite client has no connection")

func New(client sqliteclient.Client) (Repository, error) {
	if client.DB == nil {
		return Repository{}, ErrNilClient
	}
	return Repository{conn: client.DB, queries: db.New(client.DB)}, nil
}
