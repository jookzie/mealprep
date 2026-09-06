package openfoodfacts

import (
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"time"
)

var ErrInvalidBaseURL = errors.New("invalid open food facts base url")

func New(baseURL, searchURL string) (Client, error) {
	base, err := parse(baseURL)
	if err != nil {
		return Client{}, err
	}

	search, err := parse(searchURL)
	if err != nil {
		return Client{}, err
	}

	return Client{
		baseURL:   base,
		searchURL: search,
		http:      &http.Client{Timeout: 15 * time.Second},
	}, nil
}

func parse(raw string) (string, error) {
	parsed, err := url.Parse(raw)
	if err != nil {
		return "", errors.Join(ErrInvalidBaseURL, err)
	}
	if parsed.Scheme == "" || parsed.Host == "" {
		return "", errors.Join(ErrInvalidBaseURL, fmt.Errorf("%q has no scheme or host", raw))
	}
	return parsed.String(), nil
}
