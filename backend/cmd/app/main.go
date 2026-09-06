package main

import (
	"os"
	"os/signal"
	"syscall"

	"github.com/mertan/mealprep/internal/application"
	"github.com/rs/zerolog/log"
)

func main() {
	app, err := application.New()
	if err != nil {
		log.Fatal().Err(err).Msg("failed to create application")
	}

	go func() {
		if err = app.Start(); err != nil {
			log.Fatal().Err(err).Msg("application stopped with error")
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	if err = app.Shutdown(); err != nil {
		log.Error().Err(err).Msg("failed to shut down application")
	}
}
