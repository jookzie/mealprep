package dayplan

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/errorx"
)

func (h Handler) DeleteDayPlan(c fiber.Ctx, dayPlanID api.DayPlanId) error {
	err := h.dayPlans.Delete(c.Context(), dayPlanID)

	var notFound errorx.ErrNotFound
	switch {
	case errors.As(err, &notFound):
		return c.Status(fiber.StatusNotFound).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	return c.SendStatus(fiber.StatusNoContent)
}
