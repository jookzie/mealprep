package calendar

import (
	"errors"

	"github.com/gofiber/fiber/v3"
	openapi_types "github.com/oapi-codegen/runtime/types"

	"github.com/mertan/mealprep/internal/api"
	calendarservice "github.com/mertan/mealprep/internal/service/calendar"
)

func (h Handler) AssignCalendarDay(c fiber.Ctx, date openapi_types.Date) error {
	var request api.AssignCalendarDayRequest
	if err := c.Bind().Body(&request); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: "malformed request body"})
	}

	d, err := h.calendar.Assign(c.Context(), date.Time, request.DayPlanId)

	switch {
	case errors.Is(err, calendarservice.ErrUnknownDayPlan):
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	return c.JSON(api.AssignCalendarDayResponse{Day: api.CalendarDay{
		Date:      openapi_types.Date{Time: d.Date},
		DayPlanId: d.DayPlanID,
		CreatedAt: d.CreatedAt,
		UpdatedAt: d.UpdatedAt,
	}})
}
