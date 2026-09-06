package application

import (
	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/healthcheck"

	"github.com/mertan/mealprep/internal/api"
	mealhandler "github.com/mertan/mealprep/internal/handler/meal"
	mealservice "github.com/mertan/mealprep/internal/service/meal"
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
	server.Get(healthcheck.LivenessEndpoint, healthcheck.New())
	server.Get(healthcheck.ReadinessEndpoint, healthcheck.New())

	api.RegisterHandlers(server.Group("/v1"), mealhandler.New(mealservice.New()))

	return Application{cfg: cfg, fiber: server}, nil
}
