package meal

import (
	"errors"

	"github.com/gofiber/fiber/v3"
	openapi_types "github.com/oapi-codegen/runtime/types"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/errorx"
)

func (h Handler) GetMeal(c fiber.Ctx, mealID openapi_types.UUID) error {
	found, err := h.meals.Get(mealID)

	var notFound errorx.ErrNotFound
	switch {
	case errors.As(err, &notFound):
		return c.Status(fiber.StatusNotFound).JSON(api.ErrorResponse{Message: notFound.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	return c.JSON(api.GetMealResponse{Meal: api.Meal{
		Id:       found.ID,
		Label:    found.Label,
		Calories: found.Calories,
	}})
}
