package application

import (
	"errors"
	"os"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

var ErrInvalidLogLevel = errors.New("invalid log level")

func configureLogger(level string) error {
	lvl, err := zerolog.ParseLevel(level)
	if err != nil {
		return errors.Join(ErrInvalidLogLevel, err)
	}
	zerolog.SetGlobalLevel(lvl)
	log.Logger = zerolog.New(os.Stdout).With().Timestamp().Logger()
	return nil
}
