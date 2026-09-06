package application

import "github.com/caarlos0/env/v11"

// Config settings are required unless they declare an envDefault.
type Config struct {
	Port                   string `env:"PORT"`
	LogLevel               string `env:"LOG_LEVEL"`
	DBPath                 string `env:"DB_PATH"`
	OpenFoodFactsURL       string `env:"OPEN_FOOD_FACTS_URL" envDefault:"https://world.openfoodfacts.org"`
	OpenFoodFactsSearchURL string `env:"OPEN_FOOD_FACTS_SEARCH_URL" envDefault:"https://search.openfoodfacts.org"`
}

func parseConfig() (Config, error) {
	var cfg Config
	// RequiredIfNoDef makes every field without an envDefault required.
	if err := env.ParseWithOptions(&cfg, env.Options{RequiredIfNoDef: true}); err != nil {
		return Config{}, err
	}
	return cfg, nil
}
