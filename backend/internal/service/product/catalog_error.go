package product

import (
	"errors"

	"github.com/mertan/mealprep/internal/errorx"
)

var ErrCatalogUnavailable = errors.New("product catalog is unavailable")

// catalogError keeps a not-found answer as it is and marks every other catalog
// failure as unavailability, so handlers need no knowledge of the client.
func catalogError(err error) error {
	var notFound errorx.ErrNotFound
	if err == nil || errors.As(err, &notFound) {
		return err
	}
	return errors.Join(ErrCatalogUnavailable, err)
}
