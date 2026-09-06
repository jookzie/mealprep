package application

import (
	"github.com/mertan/mealprep/internal/client/openfoodfacts"
	sqliteclient "github.com/mertan/mealprep/internal/client/sqlite"
)

// clients holds the connections to everything outside the process.
type clients struct {
	sqlite        sqliteclient.Client
	openfoodfacts openfoodfacts.Client
}

func (c clients) from(cfg Config) (clients, error) {
	sqlite, err := sqliteclient.New(cfg.DBPath)
	if err != nil {
		return clients{}, err
	}

	catalog, err := openfoodfacts.New(cfg.OpenFoodFactsURL, cfg.OpenFoodFactsSearchURL)
	if err != nil {
		return clients{}, err
	}

	return clients{sqlite: sqlite, openfoodfacts: catalog}, nil
}

func (c clients) close() error {
	return c.sqlite.Close()
}
