package dayplan

import "errors"

var (
	ErrNilRepository = errors.New("day plan repository is nil")
	ErrNilMeals      = errors.New("meal repository is nil")
	ErrNilProducts   = errors.New("product repository is nil")
)

func New(repository Repository, meals Meals, products Products) (Service, error) {
	if repository == nil {
		return Service{}, ErrNilRepository
	}
	if meals == nil {
		return Service{}, ErrNilMeals
	}
	if products == nil {
		return Service{}, ErrNilProducts
	}
	return Service{repository: repository, meals: meals, products: products}, nil
}
