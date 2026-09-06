// Package targets serves the targets endpoints of the generated API.
package targets

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
)

type Service interface {
	Get(ctx context.Context) (domain.Targets, error)
	Set(ctx context.Context, macros domain.Macros) (domain.Targets, error)
}

type Handler struct {
	targets Service
}
