package product

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/errorx"
	productservice "github.com/mertan/mealprep/internal/service/product"
)

func (h Handler) ImportProduct(c fiber.Ctx) error {
	var request api.ImportProductRequest
	if err := c.Bind().Body(&request); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: "malformed request body"})
	}

	p, err := h.products.Import(c.Context(), request.Code)

	var notFound errorx.ErrNotFound
	switch {
	case errors.Is(err, productservice.ErrIncompleteProduct):
		return c.Status(fiber.StatusUnprocessableEntity).JSON(api.ErrorResponse{Message: err.Error()})
	case errors.Is(err, productservice.ErrInvalidProduct):
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: err.Error()})
	case errors.As(err, &notFound):
		return c.Status(fiber.StatusNotFound).JSON(api.ErrorResponse{Message: err.Error()})
	case errors.Is(err, productservice.ErrCatalogUnavailable):
		return c.Status(fiber.StatusBadGateway).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	var brand *string
	if p.Brand != "" {
		brand = &p.Brand
	}
	sourceCode := p.SourceCode

	return c.Status(fiber.StatusCreated).JSON(api.ImportProductResponse{Product: api.Product{
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
		SourceCode: &sourceCode,
		CreatedAt:  p.CreatedAt,
		UpdatedAt:  p.UpdatedAt,
	}})
}
