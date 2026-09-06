package calendar

import (
	"errors"

	"github.com/gofiber/fiber/v3"
	openapi_types "github.com/oapi-codegen/runtime/types"

	"github.com/mertan/mealprep/internal/api"
	calendarservice "github.com/mertan/mealprep/internal/service/calendar"
)

func (h Handler) ListCalendarDays(c fiber.Ctx, params api.ListCalendarDaysParams) error {
	days, err := h.calendar.List(c.Context(), params.From.Time, params.To.Time)

	switch {
	case errors.Is(err, calendarservice.ErrInvalidRange):
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	response := api.ListCalendarDaysResponse{Days: make([]api.CalendarDay, 0, len(days))}
	for _, d := range days {
		response.Days = append(response.Days, api.CalendarDay{
			Date:      openapi_types.Date{Time: d.Date},
			DayPlanId: d.DayPlanID,
			CreatedAt: d.CreatedAt,
			UpdatedAt: d.UpdatedAt,
		})
	}

	return c.JSON(response)
}
