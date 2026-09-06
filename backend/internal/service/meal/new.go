package meal

import "errors"

var (
	ErrNilRepository = errors.New("meal repository is nil")
	ErrNilProducts   = errors.New("product repository is nil")
)

func New(repository Repository, products Products) (Service, error) {
	if repository == nil {
		return Service{}, ErrNilRepository
	}
	if products == nil {
		return Service{}, ErrNilProducts
	}
	return Service{repository: repository, products: products}, nil
}
