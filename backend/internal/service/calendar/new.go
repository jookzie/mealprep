package calendar

import "errors"

var (
	ErrNilRepository = errors.New("calendar repository is nil")
	ErrNilDayPlans   = errors.New("day plan repository is nil")
)

func New(repository Repository, dayPlans DayPlans) (Service, error) {
	if repository == nil {
		return Service{}, ErrNilRepository
	}
	if dayPlans == nil {
		return Service{}, ErrNilDayPlans
	}
	return Service{repository: repository, dayPlans: dayPlans}, nil
}
