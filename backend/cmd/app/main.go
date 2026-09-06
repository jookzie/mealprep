// Package main is the backend entrypoint.
package main

import (
	"context"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/rs/zerolog/log"

	"github.com/mertan/mealprep/internal/application"
)

const shutdownTimeout = 5 * time.Second

func main() {
	app, err := application.New()
	if err != nil {
		log.Fatal().Err(err).Msg("failed to create application")
	}

	go func() {
		if err := app.Start(); err != nil {
			log.Fatal().Err(err).Msg("application stopped with error")
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	ctx, cancel := context.WithTimeout(context.Background(), shutdownTimeout)
	defer cancel()

	if err := app.Shutdown(ctx); err != nil {
		log.Error().Err(err).Msg("failed to shut down application")
	}
}
