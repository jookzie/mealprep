// Package service holds what the domain services share: deriving nutrients from
// compositions, which is never persisted.
package service

import (
	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

// MealMacros derives a meal's macros from its servings. Product macros are per
// 100 units and a serving amount is in those same units.
func MealMacros(meal domain.Meal, products map[uuid.UUID]domain.Product) domain.Macros {
	var total domain.Macros
	for _, serving := range meal.Servings {
		product, ok := products[serving.ProductID]
		if !ok {
			continue
		}
		total = total.Add(product.Macros.Scale(serving.Amount / 100))
	}
	return total
}

func IndexProducts(products []domain.Product) map[uuid.UUID]domain.Product {
	index := make(map[uuid.UUID]domain.Product, len(products))
	for _, product := range products {
		index[product.ID] = product
	}
	return index
}

// ProductIDs collects the distinct products the meals refer to.
func ProductIDs(meals []domain.Meal) []uuid.UUID {
	seen := make(map[uuid.UUID]struct{})
	ids := make([]uuid.UUID, 0)
	for _, meal := range meals {
		for _, serving := range meal.Servings {
			if _, ok := seen[serving.ProductID]; ok {
				continue
			}
			seen[serving.ProductID] = struct{}{}
			ids = append(ids, serving.ProductID)
		}
	}
	return ids
}
