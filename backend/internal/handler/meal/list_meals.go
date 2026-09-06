package meal

import (
	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
)

func (h Handler) ListMeals(c fiber.Ctx) error {
	meals, err := h.meals.List(c.Context())
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	response := api.ListMealsResponse{Meals: make([]api.Meal, 0, len(meals))}
	for _, m := range meals {
		servings := make([]api.Serving, 0, len(m.Servings))
		for _, s := range m.Servings {
			servings = append(servings, api.Serving{ProductId: s.ProductID, Amount: s.Amount})
		}
		response.Meals = append(response.Meals, api.Meal{
			Id:       m.ID,
			Label:    m.Label,
			Servings: servings,
			Macros: api.Macros{
				EnergyKcal:     m.Macros.EnergyKcal,
				FatG:           m.Macros.FatG,
				ProteinG:       m.Macros.ProteinG,
				CarbohydratesG: m.Macros.CarbohydratesG,
			},
			CreatedAt: m.CreatedAt,
			UpdatedAt: m.UpdatedAt,
		})
	}

	return c.JSON(response)
}
