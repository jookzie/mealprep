package product

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/errorx"
)

func (h Handler) GetProduct(c fiber.Ctx, productID api.ProductId) error {
	p, err := h.products.Get(c.Context(), productID)

	var notFound errorx.ErrNotFound
	switch {
	case errors.As(err, &notFound):
		return c.Status(fiber.StatusNotFound).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	var sourceCode *string
	if p.SourceCode != "" {
		sourceCode = &p.SourceCode
	}

	return c.JSON(api.GetProductResponse{Product: api.Product{
		Id:   p.ID,
		Name: p.Name,
		Unit: api.Unit(p.Unit),
		Macros: api.Macros{
			EnergyKcal:     p.Macros.EnergyKcal,
			FatG:           p.Macros.FatG,
			ProteinG:       p.Macros.ProteinG,
			CarbohydratesG: p.Macros.CarbohydratesG,
		},
		Nutrients:  p.Nutrients,
		SourceCode: sourceCode,
		CreatedAt:  p.CreatedAt,
		UpdatedAt:  p.UpdatedAt,
	}})
}
