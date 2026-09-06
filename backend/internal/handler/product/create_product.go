package product

import (
	"errors"

	"github.com/gofiber/fiber/v3"

	"github.com/mertan/mealprep/internal/api"
	"github.com/mertan/mealprep/internal/domain"
	productservice "github.com/mertan/mealprep/internal/service/product"
)

func (h Handler) CreateProduct(c fiber.Ctx) error {
	var request api.CreateProductRequest
	if err := c.Bind().Body(&request); err != nil {
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: "malformed request body"})
	}

	var nutrients map[string]float64
	if request.Nutrients != nil {
		nutrients = *request.Nutrients
	}

	p, err := h.products.Create(c.Context(), domain.Product{
		Name: request.Name,
		Unit: domain.Unit(request.Unit),
		Macros: domain.Macros{
			EnergyKcal:     request.Macros.EnergyKcal,
			FatG:           request.Macros.FatG,
			ProteinG:       request.Macros.ProteinG,
			CarbohydratesG: request.Macros.CarbohydratesG,
		},
		Nutrients: nutrients,
	})

	switch {
	case errors.Is(err, productservice.ErrInvalidProduct):
		return c.Status(fiber.StatusBadRequest).JSON(api.ErrorResponse{Message: err.Error()})
	case err != nil:
		return c.Status(fiber.StatusInternalServerError).JSON(api.ErrorResponse{Message: "internal error"})
	}

	return c.Status(fiber.StatusCreated).JSON(api.CreateProductResponse{Product: api.Product{
		Id:   p.ID,
		Name: p.Name,
		Unit: api.Unit(p.Unit),
		Macros: api.Macros{
			EnergyKcal:     p.Macros.EnergyKcal,
			FatG:           p.Macros.FatG,
			ProteinG:       p.Macros.ProteinG,
			CarbohydratesG: p.Macros.CarbohydratesG,
		},
		Nutrients: p.Nutrients,
		CreatedAt: p.CreatedAt,
		UpdatedAt: p.UpdatedAt,
	}})
}
