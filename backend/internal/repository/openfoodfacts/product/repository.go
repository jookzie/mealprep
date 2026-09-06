// Package product reads products from the Open Food Facts catalog.
package product

import "github.com/mertan/mealprep/internal/client/openfoodfacts"

type Repository struct {
	client openfoodfacts.Client
}
