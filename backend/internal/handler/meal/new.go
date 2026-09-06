package meal

import "errors"

var ErrNilService = errors.New("meal service is nil")

func New(meals Service) (Handler, error) {
	if meals == nil {
		return Handler{}, ErrNilService
	}
	return Handler{meals: meals}, nil
}
