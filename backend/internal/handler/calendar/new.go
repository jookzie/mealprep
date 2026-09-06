package calendar

import "errors"

var ErrNilService = errors.New("calendar service is nil")

func New(calendar Service) (Handler, error) {
	if calendar == nil {
		return Handler{}, ErrNilService
	}
	return Handler{calendar: calendar}, nil
}
