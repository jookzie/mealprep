package product

import (
	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
)

func (h Handler) ListProducts(c fiber.Ctx) error {
	products, err := h.products.List(c.Context())
	if err != nil {
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	response := api.ListProductsResponse{Products: make([]api.Product, 0, len(products))}
	for _, p := range products {
		var brand *string
		if p.Brand != "" {
			brand = &p.Brand
		}
		var sourceCode *string
		if p.SourceCode != "" {
			sourceCode = &p.SourceCode
		}
		response.Products = append(response.Products, api.Product{
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
			Brand:      brand,
			SourceCode: sourceCode,
			CreatedAt:  p.CreatedAt,
			UpdatedAt:  p.UpdatedAt,
		})
	}

	return c.JSON(response)
}
