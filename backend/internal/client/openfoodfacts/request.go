package openfoodfacts

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
)

var (
	ErrRequest = errors.New("open food facts request failed")
	ErrStatus  = errors.New("open food facts answered with an error status")
	ErrDecode  = errors.New("decode open food facts response")
)

// Open Food Facts asks every client to identify itself.
const userAgent = "mealprep/0.1 (https://github.com/mertan/mealprep)"

// rawProduct mirrors the fields requested from the API. Nutriment values arrive as
// numbers or strings, so they are decoded loosely and filtered in nutriments().
type rawProduct struct {
	Code        string `json:"code"`
	ProductName string `json:"product_name"`
	// BrandTags is preferred to the "brands" string, which is comma-separated
	// with inconsistent casing.
	BrandTags        []string       `json:"brands_tags"`
	ImageURL         string         `json:"image_front_small_url"`
	NutritionDataPer string         `json:"nutrition_data_per"`
	Nutriments       map[string]any `json:"nutriments"`
}

func (r rawProduct) product() Product {
	nutriments := make(map[string]float64)
	for key, value := range r.Nutriments {
		name, ok := strings.CutSuffix(key, "_100g")
		if !ok {
			continue
		}
		if number, ok := value.(float64); ok {
			nutriments[name] = number
		}
	}

	var brand string
	if len(r.BrandTags) > 0 {
		brand = r.BrandTags[0]
	}

	return Product{
		Code:             r.Code,
		Name:             r.ProductName,
		Brand:            brand,
		ImageURL:         r.ImageURL,
		NutritionDataPer: r.NutritionDataPer,
		Nutriments:       nutriments,
	}
}

func (c Client) get(ctx context.Context, target string, out any) error {
	request, err := http.NewRequestWithContext(ctx, http.MethodGet, target, nil)
	if err != nil {
		return errors.Join(ErrRequest, err)
	}
	request.Header.Set("User-Agent", userAgent)
	request.Header.Set("Accept", "application/json")

	response, err := c.http.Do(request)
	if err != nil {
		return errors.Join(ErrRequest, err)
	}
	defer func() { _ = response.Body.Close() }()

	if response.StatusCode != http.StatusOK {
		return errors.Join(ErrStatus, fmt.Errorf("%s: %s", target, response.Status))
	}

	if err = json.NewDecoder(response.Body).Decode(out); err != nil {
		return errors.Join(ErrDecode, err)
	}

	return nil
}
