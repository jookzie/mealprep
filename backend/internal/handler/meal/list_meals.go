package meal

import (
	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
)

func (h Handler) ListMeals(c fiber.Ctx) error {
	meals := h.meals.List()

	response := api.ListMealsResponse{Meals: make([]api.Meal, 0, len(meals))}
	for _, m := range meals {
		response.Meals = append(response.Meals, api.Meal{
			Id:       m.ID,
			Label:    m.Label,
			Calories: m.Calories,
		})
	}

	return c.JSON(response)
}
