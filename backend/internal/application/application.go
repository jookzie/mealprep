package application

import (
	"errors"
	"os"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

var ErrInvalidLogLevel = errors.New("invalid log level")

// Application wires all packages together and owns the Fiber lifecycle.
type Application struct {
	cfg   Config
	fiber *fiber.App
}

// New parses config, configures the global logger and wires dependencies.
func New() (*Application, error) {
	cfg, err := parseConfig()
	if err != nil {
		return nil, err
	}

	if err := configureLogger(cfg.LogLevel); err != nil {
		return nil, err
	}

	app := &Application{
		cfg:   cfg,
		fiber: fiber.New(),
	}
	app.registerHandlers()

	return app, nil
}

// Start runs the HTTP server. Blocks until the server stops.
func (a *Application) Start() error {
	log.Info().Str("port", a.cfg.Port).Msg("starting server")
	return a.fiber.Listen(":" + a.cfg.Port)
}

// Shutdown stops the HTTP server gracefully.
func (a *Application) Shutdown() error {
	log.Info().Msg("shutting down")
	return a.fiber.ShutdownWithTimeout(5 * time.Second)
}

// registerHandlers resolves handlers and registers their routes.
func (a *Application) registerHandlers() {
	a.fiber.Get("/health", func(c *fiber.Ctx) error {
		return c.SendStatus(fiber.StatusOK)
	})
}

func configureLogger(level string) error {
	lvl, err := zerolog.ParseLevel(level)
	if err != nil {
		return errors.Join(ErrInvalidLogLevel, err)
	}
	zerolog.SetGlobalLevel(lvl)
	log.Logger = zerolog.New(os.Stdout).With().Timestamp().Logger()
	return nil
}
