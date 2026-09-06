package targets

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/domain"
	targetsservice "github.com/mertan/mealprep/internal/service/targets"
)

func (h Handler) SetTargets(c fiber.Ctx) error {
	var request api.SetTargetsRequest
	if err := c.Bind().Body(&request); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: "malformed request body"})
	}

	t, err := h.targets.Set(c.Context(), domain.Macros{
		EnergyKcal:     request.Macros.EnergyKcal,
		FatG:           request.Macros.FatG,
		ProteinG:       request.Macros.ProteinG,
		CarbohydratesG: request.Macros.CarbohydratesG,
	})

	switch {
	case errors.Is(err, targetsservice.ErrInvalidTargets):
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	return c.JSON(api.SetTargetsResponse{Targets: api.Targets{
		Macros: api.Macros{
			EnergyKcal:     t.Macros.EnergyKcal,
			FatG:           t.Macros.FatG,
			ProteinG:       t.Macros.ProteinG,
			CarbohydratesG: t.Macros.CarbohydratesG,
		},
		CreatedAt: t.CreatedAt,
		UpdatedAt: t.UpdatedAt,
	}})
}
