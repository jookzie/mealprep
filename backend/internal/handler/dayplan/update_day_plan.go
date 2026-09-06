package dayplan

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
	dayplanservice "github.com/mertan/mealprep/internal/service/dayplan"
)

func (h Handler) UpdateDayPlan(c fiber.Ctx, dayPlanID api.DayPlanId) error {
	var request api.UpdateDayPlanRequest
	if err := c.Bind().Body(&request); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: "malformed request body"})
	}

	p, err := h.dayPlans.Update(c.Context(), domain.DayPlan{ID: dayPlanID, Label: request.Label, MealIDs: request.MealIds})

	var notFound errorx.ErrNotFound
	switch {
	case errors.Is(err, dayplanservice.ErrInvalidDayPlan), errors.Is(err, dayplanservice.ErrUnknownMeal):
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: err.Error()})
	case errors.As(err, &notFound):
		return c.Status(fiber.StatusNotFound).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	meals := make([]api.DayPlanMeal, 0, len(p.Meals))
	for _, m := range p.Meals {
		meals = append(meals, api.DayPlanMeal{
			Id:    m.ID,
			Label: m.Label,
			Macros: api.Macros{
				EnergyKcal:     m.Macros.EnergyKcal,
				FatG:           m.Macros.FatG,
				ProteinG:       m.Macros.ProteinG,
				CarbohydratesG: m.Macros.CarbohydratesG,
			},
		})
	}

	return c.JSON(api.UpdateDayPlanResponse{DayPlan: api.DayPlan{
		Id:    p.ID,
		Label: p.Label,
		Meals: meals,
		Macros: api.Macros{
			EnergyKcal:     p.Macros.EnergyKcal,
			FatG:           p.Macros.FatG,
			ProteinG:       p.Macros.ProteinG,
			CarbohydratesG: p.Macros.CarbohydratesG,
		},
		CreatedAt: p.CreatedAt,
		UpdatedAt: p.UpdatedAt,
	}})
}
