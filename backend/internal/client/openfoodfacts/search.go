package openfoodfacts

import (
	"context"
	"net/url"
)

const fields = "code,product_name,brands_tags,image_front_small_url,nutriments,nutrition_data_per"

// Search uses the search host; the site's own search endpoint is rate-limited.
func (c Client) Search(ctx context.Context, query string) ([]Product, error) {
	params := url.Values{
		"q":         {query},
		"page_size": {"20"},
		"fields":    {fields},
	}

	var body struct {
		Hits []rawProduct `json:"hits"`
	}
	if err := c.get(ctx, c.searchURL+"/search?"+params.Encode(), &body); err != nil {
		return nil, err
	}

	products := make([]Product, 0, len(body.Hits))
	for _, hit := range body.Hits {
		products = append(products, hit.product())
	}

	return products, nil
}
