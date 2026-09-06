package dayplan

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/domain"
	dayplanservice "github.com/mertan/mealprep/internal/service/dayplan"
)

func (h Handler) CreateDayPlan(c fiber.Ctx) error {
	var request api.CreateDayPlanRequest
	if err := c.Bind().Body(&request); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: "malformed request body"})
	}

	p, err := h.dayPlans.Create(c.Context(), domain.DayPlan{Label: request.Label, MealIDs: request.MealIds})

	switch {
	case errors.Is(err, dayplanservice.ErrInvalidDayPlan), errors.Is(err, dayplanservice.ErrUnknownMeal):
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: err.Error()})
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

	return c.Status(fiber.StatusCreated).JSON(api.CreateDayPlanResponse{DayPlan: api.DayPlan{
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
