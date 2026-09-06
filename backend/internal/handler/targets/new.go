package targets

import "errors"

var ErrNilService = errors.New("targets service is nil")

func New(targets Service) (Handler, error) {
	if targets == nil {
		return Handler{}, ErrNilService
	}
	return Handler{targets: targets}, nil
}
