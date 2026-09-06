// Package openfoodfacts talks to the Open Food Facts API.
package openfoodfacts

import "net/http"

// Products are fetched from the main site; search lives on its own host, so the
// client holds both base URLs.
type Client struct {
	baseURL   string
	searchURL string
	http      *http.Client
}

// Product is an entry as Open Food Facts returns it. Nutriments is keyed by the
// API's own names with the "_100g" suffix removed, and holds only numeric values.
type Product struct {
	Code             string
	Name             string
	NutritionDataPer string
	Nutriments       map[string]float64
}
