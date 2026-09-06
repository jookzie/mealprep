package product

import "errors"

var ErrNilService = errors.New("product service is nil")

func New(products Service) (Handler, error) {
	if products == nil {
		return Handler{}, ErrNilService
	}
	return Handler{products: products}, nil
}
