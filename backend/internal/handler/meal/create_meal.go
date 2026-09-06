package meal

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	mealservice "github.com/mertan/mealprep/internal/service/meal"
)

func (h Handler) CreateMeal(c fiber.Ctx) error {
	var request api.CreateMealRequest
	if err := c.Bind().Body(&request); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: "malformed request body"})
	}

	created, err := h.meals.Create(request.Label, request.Calories)

	switch {
	case errors.Is(err, mealservice.ErrInvalidMeal):
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	return c.Status(fiber.StatusCreated).JSON(api.CreateMealResponse{Meal: api.Meal{
		Id:       created.ID,
		Label:    created.Label,
		Calories: created.Calories,
	}})
}
