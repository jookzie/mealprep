package product

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	productservice "github.com/mertan/mealprep/internal/service/product"
)

func (h Handler) SearchProducts(c fiber.Ctx, params api.SearchProductsParams) error {
	entries, err := h.products.Search(c.Context(), params.Q)

	switch {
	case errors.Is(err, productservice.ErrEmptyQuery):
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: err.Error()})
	case errors.Is(err, productservice.ErrCatalogUnavailable):
		return c.Status(fiber.StatusBadGateway).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	response := api.SearchProductsResponse{Entries: make([]api.CatalogEntry, 0, len(entries))}
	for _, e := range entries {
		response.Entries = append(response.Entries, api.CatalogEntry{
			Code: e.Code,
			Name: e.Name,
			Unit: api.Unit(e.Unit),
			Macros: api.Macros{
				EnergyKcal:     e.Macros.EnergyKcal,
				FatG:           e.Macros.FatG,
				ProteinG:       e.Macros.ProteinG,
				CarbohydratesG: e.Macros.CarbohydratesG,
			},
			Nutrients: e.Nutrients,
			Complete:  e.Complete,
		})
	}

	return c.JSON(response)
}
