package targets

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/errorx"
)

func (h Handler) GetTargets(c fiber.Ctx) error {
	t, err := h.targets.Get(c.Context())

	var notFound errorx.ErrNotFound
	switch {
	case errors.As(err, &notFound):
		return c.Status(fiber.StatusNotFound).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	return c.JSON(api.GetTargetsResponse{Targets: api.Targets{
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
