package dayplan

import "errors"

var ErrNilService = errors.New("day plan service is nil")

func New(dayPlans Service) (Handler, error) {
	if dayPlans == nil {
		return Handler{}, ErrNilService
	}
	return Handler{dayPlans: dayPlans}, nil
}
