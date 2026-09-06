// Package application wires the packages together and owns the Fiber lifecycle.
package application

import (
	"context"

	"github.com/gofiber/fiber/v3"
	"github.com/rs/zerolog/log"
)

type Application struct {
	cfg   Config
	fiber *fiber.App
}

// Start blocks until the server stops.
func (a Application) Start() error {
	log.Info().Str("port", a.cfg.Port).Msg("starting server")
	return a.fiber.Listen(":" + a.cfg.Port)
}

// Shutdown stops the server gracefully, forcing it down once ctx is done.
func (a Application) Shutdown(ctx context.Context) error {
	log.Info().Msg("shutting down")
	return a.fiber.ShutdownWithContext(ctx)
}
