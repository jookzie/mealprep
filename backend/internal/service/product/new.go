package product

import "errors"

var (
	ErrNilRepository = errors.New("product repository is nil")
	ErrNilCatalog    = errors.New("product catalog is nil")
)

func New(repository Repository, catalog Catalog) (Service, error) {
	if repository == nil {
		return Service{}, ErrNilRepository
	}
	if catalog == nil {
		return Service{}, ErrNilCatalog
	}
	return Service{repository: repository, catalog: catalog}, nil
}
