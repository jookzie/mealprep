package meal

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/errorx"
)

func (h Handler) GetMeal(c fiber.Ctx, mealID api.MealId) error {
	m, err := h.meals.Get(c.Context(), mealID)

	var notFound errorx.ErrNotFound
	switch {
	case errors.As(err, &notFound):
		return c.Status(fiber.StatusNotFound).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	servings := make([]api.Serving, 0, len(m.Servings))
	for _, s := range m.Servings {
		servings = append(servings, api.Serving{ProductId: s.ProductID, Amount: s.Amount})
	}

	return c.JSON(api.GetMealResponse{Meal: api.Meal{
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
	}})
}
