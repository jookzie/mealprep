package application

import (
	calendarservice "github.com/mertan/mealprep/internal/service/calendar"
	dayplanservice "github.com/mertan/mealprep/internal/service/dayplan"
	mealservice "github.com/mertan/mealprep/internal/service/meal"
	productservice "github.com/mertan/mealprep/internal/service/product"
	targetsservice "github.com/mertan/mealprep/internal/service/targets"
)

// services holds the domain logic, each built on the repositories it needs.
type services struct {
	product  productservice.Service
	meal     mealservice.Service
	dayPlan  dayplanservice.Service
	calendar calendarservice.Service
	targets  targetsservice.Service
}

func (s services) from(repositories repositories) (services, error) {
	product, err := productservice.New(repositories.product, repositories.catalog)
	if err != nil {
		return services{}, err
	}

	meal, err := mealservice.New(repositories.meal, repositories.product)
	if err != nil {
		return services{}, err
	}

	dayPlan, err := dayplanservice.New(repositories.dayPlan, repositories.meal, repositories.product)
	if err != nil {
		return services{}, err
	}

	calendar, err := calendarservice.New(repositories.calendar, repositories.dayPlan)
	if err != nil {
		return services{}, err
	}

	targets, err := targetsservice.New(repositories.targets)
	if err != nil {
		return services{}, err
	}

	return services{
		product:  product,
		meal:     meal,
		dayPlan:  dayPlan,
		calendar: calendar,
		targets:  targets,
	}, nil
}
