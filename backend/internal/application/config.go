package application

import "github.com/caarlos0/env/v11"

// Config holds all environment-driven settings.
type Config struct {
	Port     string `env:"PORT" envDefault:"8080"`
	LogLevel string `env:"LOG_LEVEL" envDefault:"info"`
	DBPath   string `env:"DB_PATH" envDefault:"mealprep.db"`
}

func parseConfig() (Config, error) {
	var cfg Config
	if err := env.Parse(&cfg); err != nil {
		return Config{}, err
	}
	return cfg, nil
}
