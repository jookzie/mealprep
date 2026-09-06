// Package application wires the packages together and owns the Fiber lifecycle.
package application

import (
	"context"
	"errors"

	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/healthcheck"
	"github.com/rs/zerolog/log"

	"github.com/mertan/mealprep/internal/api"
)

type Application struct {
	cfg     Config
	clients clients
	fiber   *fiber.App
}

func New() (Application, error) {
	cfg, err := parseConfig()
	if err != nil {
		return Application{}, err
	}

	if err = configureLogger(cfg.LogLevel); err != nil {
		return Application{}, err
	}

	var (
		clients      clients
		repositories repositories
		services     services
		handlers     handlers
	)

	if clients, err = clients.from(cfg); err != nil {
		return Application{}, err
	}
	if repositories, err = repositories.from(clients); err != nil {
		return Application{}, err
	}
	if services, err = services.from(repositories); err != nil {
		return Application{}, err
	}
	if handlers, err = handlers.from(services); err != nil {
		return Application{}, err
	}

	server := fiber.New()
	server.Get(healthcheck.LivenessEndpoint, healthcheck.New())
	server.Get(healthcheck.ReadinessEndpoint, healthcheck.New())

	api.RegisterHandlers(server.Group("/v1"), handlers)

	return Application{cfg: cfg, clients: clients, fiber: server}, nil
}

// Start blocks until the server stops.
func (a Application) Start() error {
	log.Info().Str("port", a.cfg.Port).Msg("starting server")
	return a.fiber.Listen(":" + a.cfg.Port)
}

// Shutdown stops the server gracefully, forcing it down once ctx is done, then
// closes the clients.
func (a Application) Shutdown(ctx context.Context) error {
	log.Info().Msg("shutting down")
	return errors.Join(a.fiber.ShutdownWithContext(ctx), a.clients.close())
}
