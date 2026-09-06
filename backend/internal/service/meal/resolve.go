package meal

import (
	"context"
	"errors"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
	"github.com/mertan/mealprep/internal/service"
)

var ErrUnknownProduct = errors.New("meal refers to a product that does not exist")

// resolve loads the products the meals refer to. With check set, a missing
// product is an error rather than a serving to skip.
func (s Service) resolve(ctx context.Context, meals []domain.Meal, check bool) (map[uuid.UUID]domain.Product, error) {
	ids := service.ProductIDs(meals)
	products, err := s.products.ListByIDs(ctx, ids)
	if err != nil {
		return nil, err
	}
	index := service.IndexProducts(products)

	if check {
		for _, id := range ids {
			if _, ok := index[id]; !ok {
				return nil, errors.Join(ErrUnknownProduct, errorx.NotFound("product", id.String(), nil))
			}
		}
	}

	return index, nil
}

func (s Service) derive(ctx context.Context, meals []domain.Meal) ([]domain.Meal, error) {
	products, err := s.resolve(ctx, meals, false)
	if err != nil {
		return nil, err
	}
	for i := range meals {
		meals[i].Macros = service.MealMacros(meals[i], products)
	}
	return meals, nil
}
