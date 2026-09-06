package application

import (
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/healthcheck"
)

func New() (Application, error) {
	cfg, err := parseConfig()
	if err != nil {
		return Application{}, err
	}

	if err = configureLogger(cfg.LogLevel); err != nil {
		return Application{}, err
	}

	server := fiber.New()
	server.Use(healthcheck.New())

	// Versioned API root; handlers register their routes onto it as they land.
	server.Group("v1")

	return Application{cfg: cfg, fiber: server}, nil
}
