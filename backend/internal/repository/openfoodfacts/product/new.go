package product

import (
	"errors"

	"github.com/mertan/mealprep/internal/client/openfoodfacts"
)

var ErrZeroClient = errors.New("open food facts client is not configured")

func New(client openfoodfacts.Client) (Repository, error) {
	if client.BaseURL() == "" {
		return Repository{}, ErrZeroClient
	}
	return Repository{client: client}, nil
}
