package product

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
	productservice "github.com/mertan/mealprep/internal/service/product"
)

func (h Handler) UpdateProduct(c fiber.Ctx, productID api.ProductId) error {
	var request api.UpdateProductRequest
	if err := c.Bind().Body(&request); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: "malformed request body"})
	}

	var nutrients map[string]float64
	if request.Nutrients != nil {
		nutrients = *request.Nutrients
	}
	var requestBrand string
	if request.Brand != nil {
		requestBrand = *request.Brand
	}

	p, err := h.products.Update(c.Context(), domain.Product{
		ID:   productID,
		Name: request.Name,
		Unit: domain.Unit(request.Unit),
		Macros: domain.Macros{
			EnergyKcal:     request.Macros.EnergyKcal,
			FatG:           request.Macros.FatG,
			ProteinG:       request.Macros.ProteinG,
			CarbohydratesG: request.Macros.CarbohydratesG,
		},
		Nutrients: nutrients,
		Brand:     requestBrand,
	})

	var notFound errorx.ErrNotFound
	switch {
	case errors.Is(err, productservice.ErrInvalidProduct):
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: err.Error()})
	case errors.As(err, &notFound):
		return c.Status(fiber.StatusNotFound).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	var brand *string
	if p.Brand != "" {
		brand = &p.Brand
	}
	var sourceCode *string
	if p.SourceCode != "" {
		sourceCode = &p.SourceCode
	}

	return c.JSON(api.UpdateProductResponse{Product: api.Product{
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
	}})
}
