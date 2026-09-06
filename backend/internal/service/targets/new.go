package targets

import "errors"

var ErrNilRepository = errors.New("targets repository is nil")

func New(repository Repository) (Service, error) {
	if repository == nil {
		return Service{}, ErrNilRepository
	}
	return Service{repository: repository}, nil
}
