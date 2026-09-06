package dayplan

import (
	"context"
	"errors"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
	"github.com/mertan/mealprep/internal/service"
)

var ErrUnknownMeal = errors.New("day plan refers to a meal that does not exist")

// resolve loads the meals the plans refer to, with their macros derived. With
// check set, a missing meal is an error rather than an entry to skip.
func (s Service) resolve(ctx context.Context, plans []domain.DayPlan, check bool) (map[uuid.UUID]domain.Meal, error) {
	ids := mealIDs(plans)
	meals, err := s.meals.ListByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}

	products, err := s.products.ListByIDs(ctx, service.ProductIDs(meals))
	if err != nil {
		return nil, err
	}
	index := service.IndexProducts(products)

	byID := make(map[uuid.UUID]domain.Meal, len(meals))
	for _, meal := range meals {
		meal.Macros = service.MealMacros(meal, index)
		byID[meal.ID] = meal
	}

	if check {
		for _, id := range ids {
			if _, ok := byID[id]; !ok {
				return nil, errors.Join(ErrUnknownMeal, errorx.NotFound("meal", id.String(), nil))
			}
		}
	}

	return byID, nil
}

func (s Service) derive(ctx context.Context, plans []domain.DayPlan) ([]domain.DayPlan, error) {
	meals, err := s.resolve(ctx, plans, false)
	if err != nil {
		return nil, err
	}

	for i := range plans {
		plans[i].Meals = make([]domain.Meal, 0, len(plans[i].MealIDs))
		plans[i].Macros = domain.Macros{}
		for _, id := range plans[i].MealIDs {
			meal, ok := meals[id]
			if !ok {
				continue
			}
			plans[i].Meals = append(plans[i].Meals, meal)
			plans[i].Macros = plans[i].Macros.Add(meal.Macros)
		}
	}

	return plans, nil
}

func mealIDs(plans []domain.DayPlan) []uuid.UUID {
	seen := make(map[uuid.UUID]struct{})
	ids := make([]uuid.UUID, 0)
	for _, plan := range plans {
		for _, id := range plan.MealIDs {
			if _, ok := seen[id]; ok {
				continue
			}
			seen[id] = struct{}{}
			ids = append(ids, id)
		}
	}
	return ids
}
