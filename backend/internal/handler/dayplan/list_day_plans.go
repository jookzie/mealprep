package dayplan

import (
	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
)

func (h Handler) ListDayPlans(c fiber.Ctx) error {
	plans, err := h.dayPlans.List(c.Context())
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	response := api.ListDayPlansResponse{DayPlans: make([]api.DayPlan, 0, len(plans))}
	for _, p := range plans {
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
		response.DayPlans = append(response.DayPlans, api.DayPlan{
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
		})
	}

	return c.JSON(response)
}
