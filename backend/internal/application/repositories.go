package application

import (
	catalogrepository "github.com/mertan/mealprep/internal/repository/openfoodfacts/product"
	calendarrepository "github.com/mertan/mealprep/internal/repository/sqlite/calendar"
	dayplanrepository "github.com/mertan/mealprep/internal/repository/sqlite/dayplan"
	mealrepository "github.com/mertan/mealprep/internal/repository/sqlite/meal"
	productrepository "github.com/mertan/mealprep/internal/repository/sqlite/product"
	targetsrepository "github.com/mertan/mealprep/internal/repository/sqlite/targets"
)

// repositories holds the persistence and catalog access, each built on a client.
type repositories struct {
	product  productrepository.Repository
	meal     mealrepository.Repository
	dayPlan  dayplanrepository.Repository
	calendar calendarrepository.Repository
	targets  targetsrepository.Repository
	catalog  catalogrepository.Repository
}

func (r repositories) from(clients clients) (repositories, error) {
	product, err := productrepository.New(clients.sqlite)
	if err != nil {
		return repositories{}, err
	}

	meal, err := mealrepository.New(clients.sqlite)
	if err != nil {
		return repositories{}, err
	}

	dayPlan, err := dayplanrepository.New(clients.sqlite)
	if err != nil {
		return repositories{}, err
	}

	calendar, err := calendarrepository.New(clients.sqlite)
	if err != nil {
		return repositories{}, err
	}

	targets, err := targetsrepository.New(clients.sqlite)
	if err != nil {
		return repositories{}, err
	}

	catalog, err := catalogrepository.New(clients.openfoodfacts)
	if err != nil {
		return repositories{}, err
	}

	return repositories{
		product:  product,
		meal:     meal,
		dayPlan:  dayPlan,
		calendar: calendar,
		targets:  targets,
		catalog:  catalog,
	}, nil
}
