package application

import (
	calendarhandler "github.com/mertan/mealprep/internal/handler/calendar"
	dayplanhandler "github.com/mertan/mealprep/internal/handler/dayplan"
	mealhandler "github.com/mertan/mealprep/internal/handler/meal"
	producthandler "github.com/mertan/mealprep/internal/handler/product"
	targetshandler "github.com/mertan/mealprep/internal/handler/targets"
)

// Aliases give each embedded handler a distinct field name, so one value carries
// every operation of the generated ServerInterface.
type (
	productHandler  = producthandler.Handler
	mealHandler     = mealhandler.Handler
	dayPlanHandler  = dayplanhandler.Handler
	calendarHandler = calendarhandler.Handler
	targetsHandler  = targetshandler.Handler
)

// handlers implements api.ServerInterface across all domains.
type handlers struct {
	productHandler
	mealHandler
	dayPlanHandler
	calendarHandler
	targetsHandler
}

func (h handlers) from(services services) (handlers, error) {
	product, err := producthandler.New(services.product)
	if err != nil {
		return handlers{}, err
	}

	meal, err := mealhandler.New(services.meal)
	if err != nil {
		return handlers{}, err
	}

	dayPlan, err := dayplanhandler.New(services.dayPlan)
	if err != nil {
		return handlers{}, err
	}

	calendar, err := calendarhandler.New(services.calendar)
	if err != nil {
		return handlers{}, err
	}

	targets, err := targetshandler.New(services.targets)
	if err != nil {
		return handlers{}, err
	}

	return handlers{
		productHandler:  product,
		mealHandler:     meal,
		dayPlanHandler:  dayPlan,
		calendarHandler: calendar,
		targetsHandler:  targets,
	}, nil
}
