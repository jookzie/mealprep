package dayplan

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/errorx"
)

func (h Handler) GetDayPlan(c fiber.Ctx, dayPlanID api.DayPlanId) error {
	p, err := h.dayPlans.Get(c.Context(), dayPlanID)

	var notFound errorx.ErrNotFound
	switch {
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

	return c.JSON(api.GetDayPlanResponse{DayPlan: api.DayPlan{
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
