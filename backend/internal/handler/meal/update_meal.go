package meal

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
	mealservice "github.com/mertan/mealprep/internal/service/meal"
)

func (h Handler) UpdateMeal(c fiber.Ctx, mealID api.MealId) error {
	var request api.UpdateMealRequest
	if err := c.Bind().Body(&request); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: "malformed request body"})
	}

	draft := domain.Meal{ID: mealID, Label: request.Label, Servings: make([]domain.Serving, 0, len(request.Servings))}
	for _, s := range request.Servings {
		draft.Servings = append(draft.Servings, domain.Serving{ProductID: s.ProductId, Amount: s.Amount})
	}

	m, err := h.meals.Update(c.Context(), draft)

	var notFound errorx.ErrNotFound
	switch {
	case errors.Is(err, mealservice.ErrInvalidMeal), errors.Is(err, mealservice.ErrUnknownProduct):
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: err.Error()})
	case errors.As(err, &notFound):
		return c.Status(fiber.StatusNotFound).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	servings := make([]api.Serving, 0, len(m.Servings))
	for _, s := range m.Servings {
		servings = append(servings, api.Serving{ProductId: s.ProductID, Amount: s.Amount})
	}

	return c.JSON(api.UpdateMealResponse{Meal: api.Meal{
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
