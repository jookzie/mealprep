package calendar

import (
	"errors"

	"github.com/gofiber/fiber/v3"
	openapi_types "github.com/oapi-codegen/runtime/types"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/errorx"
)

func (h Handler) UnassignCalendarDay(c fiber.Ctx, date openapi_types.Date) error {
	err := h.calendar.Unassign(c.Context(), date.Time)

	var notFound errorx.ErrNotFound
	switch {
	case errors.As(err, &notFound):
		return c.Status(fiber.StatusNotFound).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	return c.SendStatus(fiber.StatusNoContent)
}
