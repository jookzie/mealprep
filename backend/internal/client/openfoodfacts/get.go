package openfoodfacts

import (
	"context"
	"net/url"

	"github.com/mertan/mealprep/internal/errorx"
)

func (c Client) Get(ctx context.Context, code string) (Product, error) {
	var body struct {
		Status  int        `json:"status"`
		Product rawProduct `json:"product"`
	}

	target := c.baseURL + "/api/v2/product/" + url.PathEscape(code) + "?fields=" + fields
	if err := c.get(ctx, target, &body); err != nil {
		return Product{}, err
	}

	if body.Status != 1 {
		return Product{}, errorx.NotFound("open food facts product", code, nil)
	}

	return body.Product.product(), nil
}
